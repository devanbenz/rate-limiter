use config::FileFormat;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configuration {
    pub application_port: u16,
    pub redis_host: String,
}

pub fn get_config() -> Result<Configuration, config::ConfigError> {
    let configuration = config::Config::builder()
        .add_source(config::File::new("configuration.yaml", FileFormat::Yaml))
        .build()?;

    Ok(configuration.try_deserialize::<Configuration>()?)
}
