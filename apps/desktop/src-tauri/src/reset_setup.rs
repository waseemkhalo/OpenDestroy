//! Explicit local setup reset operations.
//!
//! These commands affect only this installation's local state. They never
//! call provider APIs and never delete cloud/account data.

use serde::Serialize;

const LOCAL_FILES: [&str; 5] = [
    "connection.json",
    "public-local-data.json",
    "public-local-model.json",
    "shortcut.txt",
    "settings-window.json",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResetResult {
    pub reset: bool,
    pub models_removed: bool,
}

fn remove_known_file(path: std::path::PathBuf, label: &str) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(format!("Cannot clear {label}")),
    }
}

fn require_confirmation(confirmation: bool) -> Result<(), String> {
    if confirmation {
        Ok(())
    } else {
        Err("Explicit confirmation is required to delete local data".into())
    }
}

/// Return to onboarding while retaining local content, settings, and saved
/// speech selection. The UI reruns setup and may explicitly choose a new
/// provider; this command does not silently discard that choice.
#[tauri::command]
pub(crate) fn public_restart_onboarding(app: tauri::AppHandle) -> Result<ResetResult, String> {
    crate::local_models::cancel_all_downloads();
    crate::auth::invalidate_session(&app)?;
    Ok(ResetResult {
        reset: true,
        models_removed: false,
    })
}

/// Delete this installation's local data. `confirmation` is intentionally
/// required so the UI cannot accidentally turn a render/retry into deletion.
/// The optional model deletion is limited to the immutable local model catalog.
#[tauri::command]
pub(crate) fn public_reset_local_data(
    app: tauri::AppHandle,
    confirmation: bool,
    remove_installed_models: bool,
) -> Result<ResetResult, String> {
    require_confirmation(confirmation)?;

    // Do this before any credential, session, or file mutation. A model lease
    // can outlive cancellation briefly, so remove_all_managed_models must not
    // be the first busy check after destructive work has started.
    if remove_installed_models {
        crate::local_models::ensure_no_model_in_use()?;
    }

    crate::local_models::cancel_all_downloads();
    let result = (|| -> Result<(), String> {
        crate::auth::boundary(&app)?;
        crate::public_setup::clear_local_speech_credentials()?;
        crate::integrations::clear_local_credentials()?;

        if remove_installed_models {
            crate::local_models::remove_all_managed_models()?;
        }
        let labels = [
            "saved connection",
            "local dictation data",
            "local model selection",
            "shortcut setting",
            "window setting",
        ];
        for (filename, label) in LOCAL_FILES.iter().zip(labels) {
            remove_known_file(
                crate::app_identity::config_path().with_file_name(filename),
                label,
            )?;
        }
        // Re-register the default in the running process after clearing the
        // old persisted shortcut, so reset takes effect without an app restart.
        crate::reset_shortcut(&app)
    })();
    result.map_err(|error| format!("Local reset partially completed: {error}"))?;

    Ok(ResetResult {
        reset: true,
        models_removed: remove_installed_models,
    })
}

#[cfg(test)]
mod tests {
    use super::{require_confirmation, LOCAL_FILES};

    #[test]
    fn deletion_requires_explicit_confirmation() {
        assert!(require_confirmation(false).is_err());
        assert!(require_confirmation(true).is_ok());
    }

    #[test]
    fn reset_plan_names_only_real_public_storage_files() {
        assert_eq!(
            LOCAL_FILES,
            [
                "connection.json",
                "public-local-data.json",
                "public-local-model.json",
                "shortcut.txt",
                "settings-window.json",
            ]
        );
        assert!(LOCAL_FILES.iter().all(|name| !name.contains("private")));
    }

    #[test]
    fn reset_plan_isolated_file_removal_is_exact() {
        let root =
            std::env::temp_dir().join(format!("destroy-reset-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("temporary reset directory");
        let target = root.join("public-local-data.json");
        let unrelated = root.join("user-notes.txt");
        std::fs::write(&target, b"test").expect("temporary local data");
        std::fs::write(&unrelated, b"keep").expect("temporary unrelated data");

        super::remove_known_file(target.clone(), "local dictation data")
            .expect("known file removal");
        assert!(!target.exists());
        assert!(unrelated.exists());
        let _ = std::fs::remove_dir_all(root);
    }
}
