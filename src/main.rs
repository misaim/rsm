
use clap::Parser;

mod ec2lite;
use crate::ec2lite::EC2Instances;
use crate::ec2lite::EC2GetMetadata;

use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{
    Dialog, 
    Panel,
    SelectView, 
};

use arboard::Clipboard;

#[derive(Debug, Parser)]
struct Opt {
    /// User supplied AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// User supplied AWS Profile.
    #[structopt(short, long)]
    profile: Option<String>,

    /// Whether to display additional runtime information. Unused.
    #[structopt(short, long)]
    _verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let Opt { region, profile, _verbose ,} = Opt::parse();

    let instances = EC2Instances::new_from_region(region, profile, EC2GetMetadata(true)).await;

    // Read from json cache file
    //let instances = ec2lite::EC2Instances::read_json("example.json");
    // Write to a json cache file
    //instances.write_json("example.json");
     
    let mut siv = cursive::default();
    
    siv.load_toml(include_str!("../style.toml")).unwrap();

    siv.add_global_callback('q', |s| s.quit());

    if instances.is_empty() {
        let no_instances_error_string = format!("No Instances were returned.\nCheck your region and AWS Credentials settings.");
        siv.add_layer(Dialog::text(no_instances_error_string)
            .title("Error")
            .button("Close", |s| s.quit() ) //Default!
        )
    }
    else {
        let mut instance_select = SelectView::new();
        instance_select.set_on_submit(show_result);
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
            Panel::new(instance_select)
                .title("Select a profile")
                .full_screen()
        );
    }

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
