use std::thread;

use bg_engine::report_submit::{
    has_leading_zero_bits, submit_report_to, ReportAttachment, ReportSubmissionRequest,
    ReportSubmitError,
};
use sha2::{Digest as _, Sha256};
use tiny_http::{Header, Response, Server, StatusCode};

fn json_response(
    status: u16,
    body: &str,
    extra_headers: Vec<Header>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut response =
        Response::from_data(body.as_bytes().to_vec()).with_status_code(StatusCode(status));
    response
        .add_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
    for header in extra_headers {
        response.add_header(header);
    }
    response
}

fn sample_request(with_zip: bool) -> ReportSubmissionRequest {
    ReportSubmissionRequest {
        client_tag: Some("0123456789abcdef0123456789abcdef".to_owned()),
        flavor: "bg".to_owned(),
        app_version: "0.1.3".to_owned(),
        kind: "problem".to_owned(),
        title: "[Problem] Installation stopped at \"Install mods\"".to_owned(),
        body: "WeiDU stopped.\n\nDetails from Chriz Easy BG:\nApp: Chriz Easy BG 0.1.3".to_owned(),
        contact: "@chrizfan".to_owned(),
        diagnostics: with_zip.then(|| ReportAttachment {
            filename: "chriz-easy-bg-diagnostics-20261113-0500.zip".to_owned(),
            bytes: b"PK\x03\x04diagnostics-payload".to_vec(),
        }),
    }
}

#[test]
fn solves_challenge_and_uploads_multipart_report_with_diagnostics_zip() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}/api/installer-report", server.server_addr());

    let handle = thread::spawn(move || {
        // 1. Challenge request
        let challenge_req = server.recv().unwrap();
        assert_eq!(challenge_req.method().as_str(), "POST");
        assert_eq!(challenge_req.url(), "/api/installer-report/challenge");
        challenge_req
            .respond(json_response(
                200,
                r#"{"status":"ok","challenge":"payload123.sig456","difficulty":8,"expiresAt":"2026-11-13T06:00:00Z"}"#,
                Vec::new(),
            ))
            .unwrap();

        // 2. Multipart submit request
        let mut submit_req = server.recv().unwrap();
        assert_eq!(submit_req.method().as_str(), "POST");
        assert_eq!(submit_req.url(), "/api/installer-report");
        let content_type = submit_req
            .headers()
            .iter()
            .find(|header| header.field.equiv("Content-Type"))
            .map(|header| header.value.as_str().to_owned())
            .expect("multipart Content-Type");
        assert!(content_type.starts_with("multipart/form-data; boundary="));

        let mut raw_body = Vec::new();
        submit_req.as_reader().read_to_end(&mut raw_body).unwrap();
        let body_lossy = String::from_utf8_lossy(&raw_body);
        assert!(body_lossy.contains("name=\"metadata\""));
        assert!(body_lossy.contains("\"flavor\":\"bg\""));
        assert!(body_lossy.contains("\"contact\":\"@chrizfan\""));
        assert!(body_lossy.contains(
            "name=\"diagnostics\"; filename=\"chriz-easy-bg-diagnostics-20261113-0500.zip\""
        ));
        assert!(raw_body
            .windows(b"PK\x03\x04diagnostics-payload".len())
            .any(|window| window == b"PK\x03\x04diagnostics-payload"));

        // Extract metadata JSON and verify the Proof-of-Work nonce!
        let json_start = body_lossy
            .find("{\"challenge\":")
            .expect("metadata JSON start");
        let json_end = body_lossy[json_start..]
            .find("\r\n")
            .map(|offset| json_start + offset)
            .expect("metadata JSON end");
        let metadata: serde_json::Value =
            serde_json::from_str(&body_lossy[json_start..json_end]).unwrap();
        let challenge = metadata["challenge"].as_str().unwrap();
        let nonce = metadata["nonce"].as_str().unwrap();
        assert_eq!(challenge, "payload123.sig456");
        let digest = Sha256::digest(format!("chriz-report-v1:{challenge}:{nonce}").as_bytes());
        assert!(has_leading_zero_bits(&digest, 8));

        submit_req
            .respond(json_response(
                200,
                r#"{"status":"sent","reportId":"r-89abcdef"}"#,
                Vec::new(),
            ))
            .unwrap();
    });

    let result = submit_report_to(&base_url, &sample_request(true)).unwrap();
    assert_eq!(result.report_id, "r-89abcdef");
    handle.join().unwrap();
}

#[test]
fn maps_unconfigured_503_challenge_to_unavailable() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}/api/installer-report", server.server_addr());

    let handle = thread::spawn(move || {
        let req = server.recv().unwrap();
        req.respond(json_response(
            503,
            r#"{"status":"unavailable","code":"report_service_unavailable"}"#,
            Vec::new(),
        ))
        .unwrap();
    });

    let err = submit_report_to(&base_url, &sample_request(false)).unwrap_err();
    assert_eq!(
        err,
        ReportSubmitError::Unavailable("report_service_unavailable".to_owned())
    );
    handle.join().unwrap();
}

#[test]
fn maps_429_retry_after_and_413_too_large_and_excessive_difficulty() {
    let server = Server::http("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}/api/installer-report", server.server_addr());

    let handle = thread::spawn(move || {
        // First attempt: 429 with Retry-After
        let c1 = server.recv().unwrap();
        c1.respond(json_response(
            200,
            r#"{"status":"ok","challenge":"c1.sig","difficulty":1,"expiresAt":"2026-11-13T06:00:00Z"}"#,
            Vec::new(),
        ))
        .unwrap();
        let s1 = server.recv().unwrap();
        s1.respond(json_response(
            429,
            r#"{"status":"error","code":"rate_limited","retryAfterSeconds":900}"#,
            vec![Header::from_bytes(&b"Retry-After"[..], &b"900"[..]).unwrap()],
        ))
        .unwrap();

        // Second attempt: 413 payload too large
        let c2 = server.recv().unwrap();
        c2.respond(json_response(
            200,
            r#"{"status":"ok","challenge":"c2.sig","difficulty":1,"expiresAt":"2026-11-13T06:00:00Z"}"#,
            Vec::new(),
        ))
        .unwrap();
        let s2 = server.recv().unwrap();
        s2.respond(json_response(
            413,
            r#"{"status":"error","code":"report_too_large"}"#,
            Vec::new(),
        ))
        .unwrap();

        // Third attempt: excessive PoW difficulty refused before solving or uploading
        let c3 = server.recv().unwrap();
        c3.respond(json_response(
            200,
            r#"{"status":"ok","challenge":"c3.sig","difficulty":25,"expiresAt":"2026-11-13T06:00:00Z"}"#,
            Vec::new(),
        ))
        .unwrap();
    });

    assert_eq!(
        submit_report_to(&base_url, &sample_request(false)).unwrap_err(),
        ReportSubmitError::RateLimited {
            retry_after_seconds: Some(900),
        }
    );
    assert_eq!(
        submit_report_to(&base_url, &sample_request(false)).unwrap_err(),
        ReportSubmitError::TooLarge
    );
    assert_eq!(
        submit_report_to(&base_url, &sample_request(false)).unwrap_err(),
        ReportSubmitError::Rejected {
            code: "excessive_challenge_difficulty".to_owned(),
        }
    );
    handle.join().unwrap();
}
