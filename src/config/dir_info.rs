use toml::Value;
use crate::error::ConfigError;

pub struct DirInfo<'a, 'b, 'c, 'd> {
    pub idx: usize,
    pub dir: &'a str,
    pub user: Option<&'b str>,
    pub group: Option<&'c str>,
    pub mode: u16,
    pub caps: Option<&'d str>,
}

impl DirInfo<'_, '_, '_, '_> {
    pub fn new(values: &[Value]) -> Result<Vec<DirInfo>, ConfigError> {
        let mut dirs = Vec::with_capacity(values.len());
        for (idx, value) in values.iter().enumerate() {
            let table = value
                .as_table()
                .ok_or(ConfigError::DirsWrongBaseType(idx))?;

            let dir = table
                .get("dir")
                .ok_or(ConfigError::DirsMissing(idx, "dir"))?
                .as_str()
                .ok_or(ConfigError::DirsWrongType(idx, "dest", "string"))?;

            let user = table
                .get("user")
                .map(|v| v.as_str().ok_or(ConfigError::DirsWrongType(idx, "user", "string")))
                .transpose()?;

            let group = table
                .get("group")
                .map(|v| v.as_str().ok_or(ConfigError::DirsWrongType(idx, "group", "string")))
                .transpose()?;

            let mode = table
                .get("mode")
                .map(|value| {
                    let modestr = value
                        .as_str()
                        .ok_or(ConfigError::DirsWrongType(idx, "mode", "string"))?;
                    let mode = u16::from_str_radix(modestr, 8)
                        .map_err(|_| ConfigError::DirsWrongType(idx, "mode", "oct-string"))?;

                    // user supplied mode is used for permissions only
                    Ok(mode & 0o777)
                })
                .transpose()?
                .unwrap_or(0o755);

            let caps = table
                .get("caps")
                .map(|v| v.as_str().ok_or(ConfigError::DirsWrongType(idx, "caps", "string")))
                .transpose()?;


            dirs.push(DirInfo {
                idx,
                dir,
                user,
                group,
                mode,
                caps,
            });

        }

        Ok(dirs)
    }

    pub fn generate_rpm_file_options(&self) -> Result<rpm::FileOptions, ConfigError> {
        let mut option = rpm::FileOptions::new(self.dir.to_string());
        option = option.mode(rpm::FileMode::dir(self.mode));
        if let Some(user) = self.user {
            option = option.user(user);
        }
        if let Some(group) = self.group {
            option = option.group(group);
        }
        if let Some(caps) = self.caps {
            option = option.caps(caps)
                .map_err(|err| ConfigError::DirsInvalidCaps(self.idx, err.into()))?;
        }
        Ok(option.into())
    }
}
