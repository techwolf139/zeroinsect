use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredSkill {
    pub name: String,
    pub path: PathBuf,
    pub source: SkillSource,
    pub has_skill_toml: bool,
    pub has_skill_md: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillSource {
    Local,
    OpenCode,
    ZeroClaw,
    Linked,
}

pub struct SkillDiscovery;

impl SkillDiscovery {
    pub fn new() -> Self {
        Self
    }

    pub fn discover_from_opencode(&self) -> Vec<DiscoveredSkill> {
        let mut skills = Vec::new();

        if let Some(home) = dirs::home_dir() {
            let opencode_skills = home.join(".config/opencode/skills");
            if opencode_skills.exists() {
                skills.extend(self.scan_skill_directory(&opencode_skills, SkillSource::OpenCode));
            }
        }

        skills
    }

    pub fn discover_from_config(&self) -> Vec<DiscoveredSkill> {
        let mut skills = Vec::new();

        let config_dirs = vec![
            dirs::config_dir().map(|p| p.join("agent/skills")),
            dirs::home_dir().map(|p| p.join(".config/agent/skills")),
            dirs::home_dir().map(|p| p.join(".agent/skills")),
        ];

        for dir_opt in config_dirs {
            if let Some(dir) = dir_opt {
                if dir.exists() {
                    skills.extend(self.scan_skill_directory(&dir, SkillSource::ZeroClaw));
                }
            }
        }

        skills
    }

    pub fn discover_from_standard_locations(&self) -> Vec<DiscoveredSkill> {
        let mut all_skills = Vec::new();

        all_skills.extend(self.discover_from_opencode());
        all_skills.extend(self.discover_from_config());

        let local_skills = PathBuf::from("./skills");
        if local_skills.exists() {
            all_skills.extend(self.scan_skill_directory(&local_skills, SkillSource::Local));
        }

        all_skills
    }

    fn scan_skill_directory(&self, dir: &Path, source: SkillSource) -> Vec<DiscoveredSkill> {
        let mut skills = Vec::new();

        if !dir.is_dir() {
            return skills;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let has_toml = path.join("SKILL.toml").exists();
                    let has_md = path.join("SKILL.md").exists();
                    let has_scripts = path.join("scripts").is_dir();

                    if has_toml || has_md || has_scripts {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                        skills.push(DiscoveredSkill {
                            name,
                            path: path.clone(),
                            source: source.clone(),
                            has_skill_toml: has_toml,
                            has_skill_md: has_md,
                        });
                    }
                }
            }
        }

        skills
    }

    pub fn link_skill(&self, source: &Path, target_dir: &Path) -> Result<PathBuf, String> {
        if !source.exists() {
            return Err(format!("Source skill does not exist: {}", source.display()));
        }

        let skill_name = source
            .file_name()
            .ok_or("Invalid skill name")?
            .to_string_lossy()
            .to_string();

        std::fs::create_dir_all(target_dir)
            .map_err(|e| format!("Failed to create target directory: {}", e))?;

        let target = target_dir.join(&skill_name);

        if target.exists() {
            return Err(format!("Skill already exists at: {}", target.display()));
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, &target)
                .map_err(|e| format!("Failed to create symlink: {}", e))?;
        }

        #[cfg(not(unix))]
        {
            if source.is_dir() {
                copy_dir_all(source, &target)
                    .map_err(|e| format!("Failed to copy directory: {}", e))?;
            } else {
                std::fs::copy(source, &target)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }

        Ok(target)
    }

    pub fn unlink_skill(&self, name: &str, skills_dir: &Path) -> Result<(), String> {
        let skill_path = skills_dir.join(name);

        if !skill_path.exists() {
            return Err(format!("Skill not found: {}", name));
        }

        if skill_path.is_symlink() {
            std::fs::remove_file(&skill_path)
                .map_err(|e| format!("Failed to remove symlink: {}", e))?;
        } else {
            if skill_path.is_dir() {
                std::fs::remove_dir_all(&skill_path)
                    .map_err(|e| format!("Failed to remove directory: {}", e))?;
            } else {
                std::fs::remove_file(&skill_path)
                    .map_err(|e| format!("Failed to remove file: {}", e))?;
            }
        }

        Ok(())
    }

    pub fn get_linked_skills_dir(&self) -> PathBuf {
        PathBuf::from("./skills")
    }

    pub fn list_linked_skills(&self) -> Vec<DiscoveredSkill> {
        let dir = self.get_linked_skills_dir();
        self.scan_skill_directory(&dir, SkillSource::Linked)
    }
}

impl Default for SkillDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(unix))]
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_from_opencode() {
        let discovery = SkillDiscovery::new();
        let skills = discovery.discover_from_opencode();

        for skill in &skills {
            println!("Found skill: {} from {:?}", skill.name, skill.source);
        }
    }

    #[test]
    fn test_discover_standard_locations() {
        let discovery = SkillDiscovery::new();
        let skills = discovery.discover_from_standard_locations();

        println!("Total skills found: {}", skills.len());
        for skill in &skills {
            println!("  - {} ({:?})", skill.name, skill.source);
        }
    }
}
