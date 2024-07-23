use mcp3208::{Channel, Mcp3208};
use std::sync::{Arc, Mutex};
use rand::Rng;
use bevy::prelude::*;
use bevy::time::Fixed;
use bevy::utils::{Instant, Duration};
use crate::ledController::{LedControllerResource, Colors, ColorSwitcher};
pub mod ledController;

#[derive(Resource, Default)]
enum GameMode {
    Warmup,
    #[default]
    SingleSense,
    MultiSense,
}

#[derive(Resource)]
struct Mcp3208Resource {
    devices: Vec<(String, Arc<Mutex<Mcp3208>>)>,
    channels: [Channel; 8],
    active_id: Option<Vec<u8>>,
}

impl Mcp3208Resource {
    fn new() -> Self {
        // Initialize each MCP3208 device
        let devices = vec![
            ("/dev/spidev1.0".to_string(), Arc::new(Mutex::new(Mcp3208::new("/dev/spidev1.0").unwrap()))),
            ("/dev/spidev1.1".to_string(), Arc::new(Mutex::new(Mcp3208::new("/dev/spidev1.1").unwrap()))),
            ("/dev/spidev1.2".to_string(), Arc::new(Mutex::new(Mcp3208::new("/dev/spidev1.2").unwrap()))),
            ("/dev/spidev0.0".to_string(), Arc::new(Mutex::new(Mcp3208::new("/dev/spidev0.0").unwrap()))),
            ("/dev/spidev0.1".to_string(), Arc::new(Mutex::new(Mcp3208::new("/dev/spidev0.1").unwrap()))),
        ];
        let channels: [Channel; 8] = [
            Channel::Ch0, Channel::Ch1, Channel::Ch2, Channel::Ch3,
            Channel::Ch4, Channel::Ch5, Channel::Ch6, Channel::Ch7,
        ];
        Mcp3208Resource { devices, channels, active_id: None }
    }
    fn read_channel(&self, device_index: usize, chnl: Channel) -> u16 {
        let device = &self.devices[device_index].1;
        let mut mcp3208 = device.lock().unwrap();
        mcp3208.read_adc_diff(chnl).unwrap()
    }

}
 
//create a component for channel
#[derive(Component)]
struct Sensor{
    channel: Channel,
    device: usize,
    value: u16,
    id: u8,
}

fn read_all_test(mcp3208: ResMut<Mcp3208Resource>) {
    //clear previous 40 lines
    println!("\x1B[2J\x1B[1;1H");
    let mut i = 0;
    for device_index in 0..mcp3208.devices.len() {
        for channel in &mcp3208.channels {
            //if on channel 0
            let value = mcp3208.read_channel(device_index, *channel);
            if value >= 200 {
                println!("Device: {}, Channel: {:?}, ADC Value: {}", device_index, channel, value);
                i = 1;
            }
            //println!("Device: {}, Channel: {:?}, ADC Value: {}", device_index, channel, value);        
        }
    }
    if i == 1 {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    else {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
fn spawn_all_sensors(mut commands: Commands, mcp3208: Res<Mcp3208Resource>){
    let mut i = 0;
    for device_index in 0..mcp3208.devices.len() {
        for channel in &mcp3208.channels {
            commands.spawn(Sensor{
                channel: *channel,
                device: device_index,
                value: 0,
                id: i,
            });
            i += 1;
            //return if more than 34 sensors
            if i >= 34 {
                return;
            }
        }
    }
}
#[derive(Component)]
struct ID{
    pub id: u8,
}

//get id component and match to spawn that sensor
fn spawn_sensor(query_id: Query<&ID>, mut commands: Commands, mcp3208: Res<Mcp3208Resource>){
    for id in query_id.iter(){
        match id.id {
            0..8 => {
                commands.spawn(Sensor{
                    channel: mcp3208.channels[id.id as usize],
                    device: 0,
                    value: 0,
                    id: id.id,
                });
            }
            8..16 => {
                commands.spawn(Sensor{
                    channel: mcp3208.channels[(id.id-8) as usize],
                    device: 1,
                    value: 0,
                    id: id.id,
                });
            }
            16..24 => {
                commands.spawn(Sensor{
                    channel: mcp3208.channels[(id.id-16) as usize],
                    device: 2,
                    value: 0,
                    id: id.id,
                });
            }
            24..32 => {
                commands.spawn(Sensor{
                    channel: mcp3208.channels[(id.id-24) as usize],
                    device: 3,
                    value: 0,
                    id: id.id,
                });
            }
            32 | 33 => {
                commands.spawn(Sensor{
                    channel: mcp3208.channels[(id.id-32) as usize],
                    device: 4,
                    value: 0,
                    id: id.id,
                });
            }
            _ => {}
        }
    }
}

fn read_sensor(mut sensor: Query<(&mut Sensor, Entity)>, mcp3208: Res<Mcp3208Resource>){
    for (mut sensor, entity) in sensor.iter_mut(){
        let value = mcp3208.read_channel(sensor.device, sensor.channel);
        sensor.value = value;
    }
}
#[derive(Resource)]
struct PrintTimer(Timer);

fn display_sensor(sensor: Query<(&Sensor, Entity)>){
    // create variable that get the current time
    let time = Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(100));
    if time.elapsed() >= Duration::from_secs(0){
        println!("\x1B[2J\x1B[1;1H");
        for (sensor, _) in sensor.iter(){
            let id = sensor.id.clone() + 1;
            println!("Sensor ID: {}, ADC Value: {}", id, sensor.value);
        }
    }
}

fn read_specific_sensor(mut query_sensor: Query<(&mut Sensor, Entity)>, mcp3208: Res<Mcp3208Resource>){
    for (mut sensor, _) in query_sensor.iter_mut(){
        let value = mcp3208.read_channel(sensor.device, sensor.channel);
        sensor.value = value;
    }
}


pub struct Warmup;

impl Plugin for Warmup {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Mcp3208Resource::new())
            .add_systems(Startup, ((getRandomID, printID), spawn_sensor).chain())
            .add_systems(Update, read_specific_sensor)
            .add_systems(Update, display_sensor);
    }
}

fn printID(query_id: Query<&ID>){
    for id in query_id.iter(){
        println!("ID: {}", id.id);
    }
}

fn getRandomID(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let val = rng.gen_range(0..34);
    commands.spawn(ID { id: val });
}

pub struct AdcPlugin;

impl Plugin for AdcPlugin{
    fn build(&self, app: &mut App){
        app.insert_resource(Mcp3208Resource::new())
            .insert_resource(GameMode::SingleSense)
            //.insert_resource(PrintTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
            .add_systems(Startup, spawn_all_sensors)
            .add_systems(Update, read_sensor)
            .add_systems(Update, display_sensor);
            //.add_systems(Update, read_all_test);
    }

}




//spawn all leds
// standby will be the default state
// it will turn a light green if a sensor has a value over 100
fn standby(sensor: Query<(&Sensor, Entity)>, mcp3208: Res<Mcp3208Resource>, mut led_controller: NonSendMut<LedControllerResource>) {
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

