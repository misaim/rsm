use std::fmt;
use std::fs;

use serde::{Deserialize, Serialize};

use aws_config::meta::region::RegionProviderChain;
use aws_config::SdkConfig;

use aws_sdk_ec2::{config::Region, Client};

#[derive(Serialize, Deserialize)]
pub struct EC2Instance {
    pub id: String,
    name: Option<String>,
    state: Option<String>,
}

impl fmt::Display for EC2Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} -> {} {}",
            self.get_name(),
            self.id,
            self.get_state()
        )
    }
}

impl EC2Instance {
    fn new(new_instance_id: &str) -> Self {
        Self {
            id: String::from(new_instance_id),
            name: None,
            state: None,
        }
    }
    fn set_name(&mut self, name: &str) {
        self.name = Some(String::from(name));
    }
    fn set_state(&mut self, state: &str) {
        self.state = Some(String::from(state));
    }
    fn get_name(&self) -> String {
        match &self.name {
            Some(name) => String::from(name),
            None => String::new(),
        }
    }
    fn get_state(&self) -> String {
        match &self.state {
            Some(name) => format!("({})", name),
            None => format!("(State Unknown)"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EC2Instances {
    instances: Vec<EC2Instance>,
    region: Option<String>,
    profile: Option<String>,
}

impl fmt::Display for EC2Instances {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();
        for instance in self.instances.iter() {
            output.push_str(format!("{}\n", instance,).as_str());
        }
        write!(f, "Instance List: \n{}", output)
    }
}

pub struct EC2GetMetadata(pub bool);

impl EC2Instances {
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
    pub fn get_region_tag(&self) -> String {
        match &self.region {
            Some(region) => format!(" --region {}", region),
            None => String::new(),
        }
    }
    pub fn get_profile_tag(&self) -> String {
        match &self.profile {
            Some(profile) => format!(" --profile {}", profile),
            None => String::new(),
        }
    }
    fn get_instance_id_vec(&self) -> Option<Vec<String>> {
        let mut instances: Vec<String> = vec![];
        for instance in self.instances.iter() {
            instances.push(instance.id.to_string())
        }
        return Some(instances);
    }
    pub fn update_metadata_id(&mut self, id: &str, name: Option<&str>, state: Option<&str>) {
        for instance in self.instances.iter_mut() {
            if instance.id == id {
                match name {
                    Some(new_name) => instance.set_name(new_name),
                    None => (),
                }
                match state {
                    Some(new_state) => instance.set_state(new_state),
                    None => (),
                }
            }
        }
    }
    fn add_instance(&mut self, instance: EC2Instance) {
        self.instances.push(instance);
    }
    pub fn iter(&self) -> impl Iterator<Item = &EC2Instance> + '_ {
        self.instances.iter()
    }
    pub fn new() -> Self {
        Self {
            instances: vec![],
            region: None,
            profile: None,
        }
    }
    // Unsafe. Expects to be able to serialise and write an EC2Instances object to json_file.
    pub fn write_json(&self, json_file: &str) {
        let serialized = serde_json::to_string(&self).unwrap();
        fs::write(json_file, &serialized).expect("Unable to write file");
    }
    // Unsafe. Expects to be able to read from json_file and deserialise to an EC2Instances object.
    pub fn read_json(json_file: &str) -> Self {
        let data = fs::read_to_string(json_file).expect("Unable to read file");
        let instances: EC2Instances = serde_json::from_str(&data).unwrap();
        return instances;
    }
    // Unsafe. Expects SOME sort of AWS config file and valid region. While attempts are made to catch some of these, this is not an AWS credential helper tool.
    pub async fn new_from_region(
        region: Option<String>,
        profile: Option<String>,
        metadata: EC2GetMetadata,
    ) -> Self {
        let mut return_instances = self::EC2Instances::new();

        return_instances.profile = profile.clone();
        return_instances.region = region.clone();

        let region_chain = RegionProviderChain::first_try(region.map(Region::new))
            .or_default_provider()
            .or_else(Region::new("us-east-1"));

        let config: SdkConfig = match &return_instances.profile {
            Some(profile_string) => {
                aws_config::from_env()
                    .profile_name(profile_string)
                    .load()
                    .await
            }
            None => aws_config::from_env().region(region_chain).load().await,
        };

        return_instances.region = match config.region() {
            Some(read_region) => Some(format!("{}", read_region)),
            None => None,
        };

        let client = Client::new(&config);

        // first fill our Ec2instances object with all instances. Currently doesn't read status (here)... funny world we're in.
        let resp = client.describe_instance_status().send().await;

        for instance_status in resp.unwrap().instance_statuses().unwrap_or_default() {
            return_instances.add_instance(EC2Instance::new(
                instance_status.instance_id().unwrap_or_default(),
            ));
        }
        // ..then keep using the same client to grab metadata, if required. Yes this is somewhat horrible.
        match metadata {
            EC2GetMetadata(true) => {
                let instance_list = return_instances.get_instance_id_vec();
                match &instance_list {
                    Some(instance_id_vec) => {
                        if !instance_id_vec.is_empty() {
                            let resp = client
                                .describe_instances()
                                .set_instance_ids(instance_list)
                                .send()
                                .await
                                .unwrap();

                            for reservation in resp.reservations().unwrap_or_default() {
                                for instance in reservation.instances().unwrap_or_default() {
                                    let instance_id = instance.instance_id().unwrap();
                                    let mut name: Option<&str> = None;
                                    for tag in instance.tags().unwrap() {
                                        match tag.key().unwrap_or_default() {
                                            "Name" => name = Some(tag.value().unwrap_or_default()),
                                            _ => (),
                                        }
                                    }
                                    // I was fucking around reading IAM profile names but don't use them :)
                                    //let profile = match instance.iam_instance_profile() {
                                    //    Some(profile_new) => Some(profile_new.arn().unwrap()),
                                    //    None => None
                                    //};
                                    let state: Option<&str> = match instance.state().unwrap().name()
                                    {
                                        Some(state_new) => Some(state_new.as_str()),
                                        None => None,
                                    };
                                    return_instances.update_metadata_id(instance_id, name, state);
                                }
                            }
                        }
                    }
                    None => (),
                }
            }
            EC2GetMetadata(false) => (),
        }
        return return_instances;
    }
}
