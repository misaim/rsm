
use clap::Parser;
//use std::fs;

pub mod ec2lite;
use crate::ec2lite::*;

use cursive::Cursive;
use cursive::views::{
    //Button, 
    Dialog, 
    Panel,
    //DummyView, 
    //EditView,              
    //LinearLayout, 
    SelectView, 
    //TextView, 
    //BoxedView
};
use cursive::traits::*;

use arboard::Clipboard;

#[derive(Debug, Parser)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The AWS Region.
    #[structopt(short, long)]
    profile: Option<String>,

    /// Whether to display additional runtime information.
    #[structopt(short, long)]
    _verbose: bool,
}


/// Lists the events of your EC2 instances in all available regions.
/// # Arguments
///
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), ()> {
//fn main() -> Result<(), ()> {
    //tracing_subscriber::fmt::init();
    let Opt { region, profile, _verbose ,} = Opt::parse();
    /* 
    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    */


    /* 
    if verbose {
        println!("EC2 client version: {}", PKG_VERSION);
        println!(
            "Region:             {}",
            region_provider.region().await.unwrap().as_ref()
        );
        println!();
    }
    */

    //let shared_config = aws_config::from_env().region(region_provider).load().await;
    //let client = Client::new(&shared_config);

    // NOTE: EC2_Specific Code here? Uncommenting this significantly increases compile time. 
    let instances = EC2Instances::new_from_region(region, profile, EC2GetMetadata(true)).await;
    //println!("{}", instances);

    // Write Json file
    //let serialized = serde_json::to_string(&instances).unwrap();
    //fs::write("instances.json", &serialized).expect("Unable to write file");
    //println!("serialized = {}", serialized);

    // Read Json file
    //let data = fs::read_to_string("instances.json").expect("Unable to read file");
    //let instances: EC2Instances = serde_json::from_str(&data).unwrap();
    /*match &instances.region {
        Some(read_region) => println!("region = {:?}", read_region),
        None => println!("region = n/a, ensure you specify it with -r <aws_region> or within ~/.aws/credentials ")
    }
    */
    //println!("deserialized = {}", &instances);
    
    
    //println!("{}", instance_list);
    //for i in instance_list.iter() {
    //    println!("{}", i);
    //}

     
    let mut siv = cursive::default();
    
    siv.load_toml(include_str!("../style.toml")).unwrap();

    let mut instance_select = SelectView::new();
    
    siv.add_global_callback('q', |s| s.quit());
    instance_select.set_on_submit(show_result);

    if instances.is_empty() {
        //println!("Didn't get any instances fuckwit.");
        let no_instances_error_string = format!("No Instances were returned.\nCheck your region and IAM settings.");
        siv.add_layer(Dialog::text(no_instances_error_string)
            .title("Error")
            .button("Close", |s| s.quit() ) //Default!
        )
    }
    else {
        for instance in instances.iter() {
            instance_select.add_item(
                format!("{}", &instance),  
                format!(
                    "aws ssm start-session --target {}{}{}", 
                    &instance.id,
                    &instances.get_profile_tag(),
                    &instances.get_region_tag(),
                )
            );
        }
        siv.add_layer(
            //BoxedView::full_width(instance_select)
            //Dialog::around(instance_select)
            Panel::new(
                instance_select
            )
                .title("Select a profile")
                .full_screen()
        );
    }

    //select.add_item();
    //siv.call_on_name("select", |view: &mut SelectView<String>| {
    //    view.add_item_str(name)
    //});
	//siv.add_global_callback('c', |s| show_result(s, "res"));
    //let mut select2 = siv.find_name::<SelectView<String>>("select").unwrap();
    //select2.add_all_str(vec!["a", "b", "c"]);
	siv.run();
    
    Ok(())
}

fn show_result(s: &mut Cursive, msg_input: &str) {
    let mut clipboard = Clipboard::new().unwrap();
    
    clipboard.set_text(msg_input).unwrap();

    s.add_layer(Dialog::text(format!("{}\n Copied to clipboard.", msg_input))
        .title("Results")
        .button("Close", |s| s.quit() ) //Default!
        .button("Back", |s| close_page(s) )
        
    )
}

fn close_page(s: &mut Cursive) {
    s.pop_layer();
}
