use rand::Rng;
use bevy::prelude::*;
use crate::led_controller::{LedControllerResource, Colors, ColorSwitcher};
use crate::sensor_controller::*;
pub mod led_controller;
pub mod sensor_controller;

#[derive(Resource, Default)]
enum GameMode {
    Warmup,
    #[default]
    SingleSense,
    MultiSense,
}

pub struct Warmup;

impl Plugin for Warmup {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Mcp3208Resource::new())
            .add_systems(Startup, ((get_random_id, print_id), spawn_sensor).chain())
            .add_systems(Update, read_specific_sensor)
            .add_systems(Update, display_sensor);
    }
}

fn print_id(query_id: Query<&ID>){
    for id in query_id.iter(){
        println!("ID: {}", id.id);
    }
}

fn get_random_id(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let val = rng.gen_range(0..34);
    commands.spawn(ID { id: val });
}

pub struct AdcPlugin;

impl Plugin for AdcPlugin{
    fn build(&self, app: &mut App){
        app.insert_resource(Mcp3208Resource::new())
            .insert_resource(GameMode::SingleSense)
            .add_systems(Startup, spawn_all_sensors)
            .add_systems(Update, read_sensor)
            .add_systems(Update, display_sensor);
    }

}


//spawn all leds
// standby will be the default state
// it will turn a light green if a sensor has a value over 100
fn standby(sensor: Query<(&Sensor, Entity)>, mut led_controller: NonSendMut<LedControllerResource>) {
    // check if any sensor has a value over 100
    for (sensor, _) in sensor.iter(){
        let value = sensor.value;
        match value {
            100..=300 => {
                led_controller.set_ring_color(sensor.id as i32, Colors::default().green);
            }
            301..=500 => {
                led_controller.set_ring_color(sensor.id as i32, Colors::default().yellow);
            }
            501..=700 => {
                led_controller.set_ring_color(sensor.id as i32, Colors::default().red);
            }
            701..=2000 => {
                led_controller.set_ring_color(sensor.id as i32, Colors::default().purple);
            }
            _ => {
                led_controller.set_ring_color(sensor.id as i32, Colors::default().clear);
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, AdcPlugin, ColorSwitcher))
        //.add_plugins((MinimalPlugins, ColorSwitcher, Warmup))
        .add_systems(Update, standby)
        .run();
}

