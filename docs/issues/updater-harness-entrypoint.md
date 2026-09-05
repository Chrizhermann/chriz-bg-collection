# Fresh Windows updater test harness exits before enumeration

Private maintenance follow-up; not a demonstrated production-app defect.

Command (PowerShell, with `CEBG_UPDATER_SETUP`, `CEBG_UPDATER_VERSION=0.1.0-alpha.10`
and `CEBG_UPDATER_CURRENT_VERSION=0.1.0-alpha.9` set):

```powershell
cargo test -p chriz-bg-app --features tauri/test --test updater_download_acceptance real_tauri_updater_downloads_and_verifies_the_signed_nsis -- --ignored --exact --nocapture
```

Two independent clean target directories compiled successfully, then Windows returned
`0xc0000139` (`STATUS_ENTRYPOINT_NOT_FOUND`) before test enumeration. The second
build disabled incremental compilation and preferred static Rust linkage. No test
assertion or Rust stack trace was produced. Stop repeating clean compilations.

The existing compiled `updater_download_acceptance-d26df3b5ef40af79.exe` still passes
against the exact new signed setup, including real Tauri loopback download, signature,
exact-byte and tamper checks. Public HTTPS download/hash/signature verification also passes.

Read-only dumpbin comparison found identical normalized static DLL/function imports
in the old passing and both new failing harnesses. None imports `chriz_bg_app_lib.dll`
or `WebView2Loader.dll`. Old/new harnesses contain MockRuntime/tauri::test markers;
the release application contains neither. Windows events did not identify the missing
dynamic entry point. Current suspicion is the harness's dynamic initialization/runtime,
not a proven production loader error; exact cause remains unresolved.

Next bounded investigation: capture the missing dynamically resolved entry with loader
diagnostics on the test harness, and fix the harness/build invocation accordingly. Do
not infer native updater apply/restart acceptance from its download test. Do not launch
or control the desktop application without renewed user permission.
