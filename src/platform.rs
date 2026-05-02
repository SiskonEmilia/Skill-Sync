#[cfg(windows)]
mod imp {
    use std::path::Path;

    pub fn create_link(target: &Path, link: &Path) -> Result<(), String> {
        if is_junction(link) {
            junction::delete(link).map_err(|e| {
                format!("failed to delete existing junction at '{}': {e}", link.display())
            })?;
            if link.exists() {
                std::fs::remove_dir(link).map_err(|e| {
                    format!("failed to remove residual dir at '{}': {e}", link.display())
                })?;
            }
        } else if link.exists() {
            if link.is_dir() {
                std::fs::remove_dir_all(link).map_err(|e| {
                    format!("failed to remove existing dir at '{}': {e}", link.display())
                })?;
            } else {
                std::fs::remove_file(link).map_err(|e| {
                    format!(
                        "failed to remove existing entry at '{}': {e}",
                        link.display()
                    )
                })?;
            }
        }

        junction::create(target, link).map_err(|e| {
            format!(
                "failed to create junction at '{}' -> '{}': {e}",
                link.display(),
                target.display()
            )
        })
    }

    pub fn remove_link(link: &Path) -> Result<(), String> {
        if !link.exists() {
            return Ok(());
        }
        if is_junction(link) {
            junction::delete(link)
                .map_err(|e| format!("failed to delete junction at '{}': {e}", link.display()))?;
            if link.exists() {
                std::fs::remove_dir(link).map_err(|e| {
                    format!("failed to remove residual dir at '{}': {e}", link.display())
                })?;
            }
            Ok(())
        } else {
            Err(format!(
                "'{}' is not a junction — refusing to delete non-junction directory",
                link.display()
            ))
        }
    }

    pub fn is_junction(path: &Path) -> bool {
        junction::get_target(path).is_ok()
    }

    pub fn read_link(path: &Path) -> Option<String> {
        junction::get_target(path)
            .ok()
            .map(|p| p.display().to_string())
    }
}

#[cfg(not(windows))]
mod imp {
    use std::path::Path;

    pub fn create_link(target: &Path, link: &Path) -> Result<(), String> {
        if link.exists() {
            if link.is_symlink() || link.is_file() {
                std::fs::remove_file(link).map_err(|e| {
                    format!(
                        "failed to remove existing entry at '{}': {e}",
                        link.display()
                    )
                })?;
            } else {
                std::fs::remove_dir_all(link).map_err(|e| {
                    format!("failed to remove existing dir at '{}': {e}", link.display())
                })?;
            }
        }

        std::os::unix::fs::symlink(target, link).map_err(|e| {
            format!(
                "failed to create symlink at '{}' -> '{}': {e}",
                link.display(),
                target.display()
            )
        })
    }

    pub fn remove_link(link: &Path) -> Result<(), String> {
        if !link.exists() {
            return Ok(());
        }
        if link.is_symlink() {
            std::fs::remove_file(link)
                .map_err(|e| format!("failed to remove symlink at '{}': {e}", link.display()))
        } else {
            Err(format!(
                "'{}' is not a symlink — refusing to delete non-symlink directory",
                link.display()
            ))
        }
    }

    pub fn is_junction(path: &Path) -> bool {
        path.is_symlink()
    }

    pub fn read_link(path: &Path) -> Option<String> {
        std::fs::read_link(path)
            .ok()
            .map(|p| p.display().to_string())
    }
}

pub use imp::*;
