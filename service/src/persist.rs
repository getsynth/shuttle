use tokio::runtime::Runtime;

use crate::{Factory, ResourceBuilder};
use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;

use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::fs;
use std::str::FromStr;

use bincode;
use bincode::serialize_into;

use shuttle_common::project::ProjectName;

use std::error::Error;



pub struct Persist;

pub struct PersistInstance {
    project_name: ProjectName,
}

impl PersistInstance {

    pub fn save<T: Serialize>(&self, key: &str, struc: T) -> Result<(), Box<dyn Error>> {
        let project_name = self.project_name.to_string();
        fs::create_dir_all(format!("shuttle_persist/{}", project_name))?;
        
        let file = File::create(format!("shuttle_persist/{}/{}.json", project_name, key))?;
        let mut writer = BufWriter::new(file);
        Ok(serialize_into(&mut writer, &struc)?)
    }

    pub fn load<T>(&self, key: &str) -> Result<T, Box<dyn Error>>
    where
        T: DeserializeOwned,
    {
        let project_name = self.project_name.to_string();
        let file = File::open(format!("shuttle_persist/{}/{}.json", project_name, key)).unwrap();
        let reader = BufReader::new(file);
        Ok(bincode::deserialize_from(reader)?)
    }

}   


#[async_trait]
impl ResourceBuilder<PersistInstance> for Persist {
    fn new() -> Self {
        Self {}
    }

    async fn build(
        self,
        factory: &mut dyn Factory, 
        _runtime: &Runtime, 
    ) -> Result<PersistInstance, crate::Error> {


        Ok(PersistInstance {project_name: factory.get_project_name()})
        
    }


}

// Create Test
#[cfg(test)]
mod tests {
    use super::*;

    // test the folder is created
    #[test]
    fn test_create_folder() {
        let project_name = ProjectName::from_str("test").unwrap();
        
        let path = format!("shuttle_persist/{}", project_name.to_string());
        assert!(fs::metadata(path).is_ok());
    }
}

