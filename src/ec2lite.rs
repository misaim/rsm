use std::fmt;
use aws_config::SdkConfig;
use serde::{Serialize, Deserialize};

 
use aws_config::{
    meta::region::RegionProviderChain, 
    //profile::ProfileFileRegionProvider,
    //profile::profile_file::*,
    //profile::*,
};
//use aws_config::profile::Profile;
use aws_sdk_ec2::{
    config::Region, 
    //meta::PKG_VERSION, 
    Client, 
    //Error
};


#[derive(Serialize, Deserialize)]
pub struct EC2Instance {
    pub id: String,
    pub name: Option<String>,
    pub state: Option<String>,
    pub profile: Option<String>,
}

impl fmt::Display for EC2Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} -> {} {}", self.get_name(), self.id, self.get_state())
    }
}

impl EC2Instance {
    fn new(new_instance_id: &str) -> Self {
        Self {
            id: String::from(new_instance_id),
            name: None,
            state: None,
            profile: None,
        }
    }
    fn set_name(&mut self, name: &str) {
        self.name = Some(String::from(name));
    }
    fn set_state(&mut self, state: &str) {
        self.state = Some(String::from(state));
    }
    fn set_profile(&mut self, profile: &str) {
        self.profile = Some(String::from(profile));
    }
    pub fn get_profile_tag(&self) -> String {
        match &self.profile {
            Some(profile) => format!(" --profile {}", profile),
            None => String::new(),
        }
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
    pub instances: Vec<EC2Instance>,
    pub region: Option<String>,
    pub profile: Option<String>,
}

impl fmt::Display for EC2Instances {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted.
        let mut output = String::new();
        for instance in self.instances.iter() {
            output.push_str(format!(
                "{}\n",
                instance,
            ).as_str());
        }
        write!(f, "List: \n{}", output)
    }
}

pub struct EC2GetMetadata(pub bool);

impl EC2Instances {
    pub fn is_full(&self) -> bool {
        !self.instances.is_empty()
    }
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
    pub fn get_region_tag (&self) -> String {
        match &self.region {
            Some(region) => format!(" --region {}", region),
            None => String::new(),
        }
    }
    pub fn get_profile_tag (&self) -> String {
        match &self.profile {
            Some(profile) => format!(" --profile {}", profile),
            None => String::new(),
        }
    } 
    pub fn get_instance_id_vec (&self) -> Option<Vec<String>> {
        let mut instances: Vec<String> = vec![];
        for instance in self.instances.iter() {
            instances.push(instance.id.to_string())
            //println!("{}", instance.id);
         }
         return Some(instances)
    }
    pub fn update_metadata_id (&mut self, id: &str, name: Option<&str>, state: Option<&str>, profile: Option<&str> ) {
        for instance in self.instances.iter_mut() {
            if instance.id == id {
                match name {
                    Some(new_name) => instance.set_name(new_name),
                    None => ()
                }
                match state {
                    Some(new_state) => instance.set_state(new_state),
                    None => ()
                }
                match profile {
                    Some(new_profile) => instance.set_profile(new_profile),
                    None => ()
                }
            }
        }
    }
    pub fn add_instance(&mut self, instance: EC2Instance) {
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
     
    pub async fn new_from_region(region: Option<String>, profile:Option<String>, metadata: EC2GetMetadata) -> Self {
        let mut return_instances = self::EC2Instances::new();
        
        return_instances.profile = profile.clone();
        return_instances.region = region.clone();
        // ec2 boilerplate. 


        let region_chain = RegionProviderChain::first_try(region.map(Region::new))
            .or_default_provider()
            .or_else(Region::new("us-west-2"));
        
        //println!("{:?}", region_chain);

        //Profile nonsense
        /*let profile_files = ProfileFiles::builder()
            .include_default_credentials_file(true)
            .build();

        let provider = ProfileFileCredentialsProvider::builder()
            .profile_files(profile_files)
            .build();
        //};
        */
        //println!("{:?}", provider);
        


        /*
        let instances_region = region_chain.region().await;
        let region_string = match instances_region {
            Some(reg_str) => {
                reg_str.to_string()
            },
            None => String::from("") // This is unsafe - but if the user doesn't supply a valid region, their config file doesn't have a region and us-west-2 is down... fuck em.  
        };
        
        return_instances.region = Some(String::from(&region_string));
        */
        // Panics on no profile being set in config file - doesn't read default. 
        let config: SdkConfig = match &return_instances.profile {
            Some(profile_string) => aws_config::from_env().profile_name(profile_string).load().await,
            None => aws_config::from_env().region(region_chain).load().await,
        };

         
        return_instances.region = match config.region() {
            Some(read_region) => Some(format!("{}",read_region)),
            None => None
        };
        

        //let config = aws_config::from_env().region(region_chain).load().await;
        let new_client = Client::new(&config);

        //println!("{:?}", config);
        // first fill our Ec2instances object with all instances. Currently doesn't read status (here)... funny world we're in.
        let resp = new_client.describe_instance_status().send().await;

        for instance_status in resp.unwrap().instance_statuses().unwrap_or_default() {
            return_instances.add_instance( EC2Instance::new(
                instance_status.instance_id().unwrap_or_default()
            ));
        }
        // ..then keep using the same client to grab metadata, if required. 
        match metadata {
            EC2GetMetadata(true) => {
                let instance_list = return_instances.get_instance_id_vec();
                match &instance_list {
                    Some(instance_id_vec) => {
                        if !instance_id_vec.is_empty() {
                            let resp = new_client
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
                                            "Name" =>  { name = Some(tag.value().unwrap_or_default()) },
                                            _ => ()
                                        }
                                    };
                                    //let mut profile: Option<&str> = None;
                                    //let profile = match instance.iam_instance_profile() {
                                    let profile = match instance.iam_instance_profile() {
                                        Some(profile_new) => Some(profile_new.arn().unwrap()),
                                        None => None
                                    };
                                    let state: Option<&str> = match instance.state().unwrap().name() {
                                        Some(state_new) => Some(state_new.as_str()),
                                        None => None
                                    };
                                    return_instances.update_metadata_id(instance_id, name, state, profile);
                                }
                            }
                        }
                    },
                    None => ()
                }
            },
            EC2GetMetadata(false) => ()
        }
        return return_instances;
    }
    
}


// Shows the events for every Region.
 // snippet-start:[ec2.rust.list-all-instance-events]
//pub async fn show_all_events(region: Option<String> ) -> EC2Instances {
    //let resp = client.describe_regions().send().await.unwrap();
 
    //for region in resp.regions.unwrap_or_default() {
    //let reg: &'static str = Box::leak(Box::from(region.region_name().unwrap()));
    //let region_provider = RegionProviderChain::default_provider().or_else(reg);
    
    //let reg: &'static str = "us-west-2";
    //let region_provider = region;

    //let z = region_provider.region().await;

    //println!();

    /*
    let z = region_chain.region().await;
    let reg_str = match z {
        Some(reg_str) => reg_str.to_string(),
        None => String::from("N/A")
    };
    println!("Instances in region {}:", reg_str);
    println!();
    */

    /* 
    let region_chain = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    let config = aws_config::from_env().region(region_chain).load().await;
    let new_client = Client::new(&config);

    let resp = new_client.describe_instance_status().send().await;
    */
    //let mut instance_list : Vec<EC2Instance> = vec![];
    
    //let mut instances = EC2Instances::from_region(region, EC2GetMetadata(true)).await;

    //let mut instances = EC2Instances::new(); // Create an empty EC2 Array
    //for instance_status in resp.unwrap().instance_statuses().unwrap_or_default() {
    //    instances.add_instance( EC2Instance::new(
    //        instance_status.instance_id().unwrap_or_default()
    //    ));
    //}

    /* 
    
    */
    //return instances;
 //}

 