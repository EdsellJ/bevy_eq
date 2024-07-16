use bevy::prelude::*;
use bevy::time::Fixed;
use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder, StripType};
use mcp3208::{Channel, Mcp3208};
use std::sync::{Arc, Mutex};
use rand::Rng;


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
    device: String,
    value: u16,
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

fn spawn_sensor(mut commands: Commands, mcp3208: Res<Mcp3208Resource>){
    let id = 0;
    match id {
        0..=8 => {
            commands.spawn(Sensor{
                channel: mcp3208.channels[id as usize],
                device: mcp3208.devices[0].0.clone(),
                value: 0,
            });
        }
        9..=16 => {
            commands.spawn(Sensor{
                channel: mcp3208.channels[(id-8) as usize],
                device: mcp3208.devices[1].0.clone(),
                value: 0,
            });
        }
        17..=24 => {
            commands.spawn(Sensor{
                channel: mcp3208.channels[(id-16) as usize],
                device: mcp3208.devices[2].0.clone(),
                value: 0,
            });
        }
        25..=32 => {
            commands.spawn(Sensor{
                channel: mcp3208.channels[(id-24) as usize],
                device: mcp3208.devices[3].0.clone(),
                value: 0,
            });
        }
        33 => {
            commands.spawn(Sensor{
                channel: mcp3208.channels[7],
                device: mcp3208.devices[4].0.clone(),
                value: 0,
            });
        }
        _ => {}
    }
}

fn read_sensor(mut sensor: Query<(&mut Sensor, Entity)>, mcp3208: Res<Mcp3208Resource>){
    for (mut sensor, entity) in sensor.iter_mut(){
        let value = mcp3208.read_channel(0, sensor.channel);
        sensor.value = value;
    }
}
#[derive(Resource)]
struct PrintTimer(Timer);

fn display_sensor(time: Res<Time>, mut timer: ResMut<PrintTimer>, sensor: Query<(&Sensor, Entity)>){
    if timer.0.tick(time.delta()).just_finished(){
        for (sensor, _) in sensor.iter(){
            println!("Device: {}, Channel: {:?}, ADC Value: {}", sensor.device, sensor.channel, sensor.value);
        }
    }
}

fn warmup(mut mcp3208: ResMut<Mcp3208Resource>){
    //TODO: activate random light and sensor combo
    // get random number between 0 and 33 inclusive
    //let mut rng = rand::thread_rng();
    //let random_number = rng.gen_range(0..=33);
    let random_number = 0;
    // poll sensor
    //spawn_sensor(random_number, commands, mcp3208);
    //read_sensor(sensor, mcp3208);
    //print sensor value
    //for (sensor, _) in sensor.iter_mut(){
        //println!("Device: {}, Channel: {:?}, ADC Value: {}", sensor.device, sensor.channel, sensor.value);
    //}

}
pub struct AdcPlugin;

impl Plugin for AdcPlugin{
    fn build(&self, app: &mut App){
        app.insert_resource(Mcp3208Resource::new())
            .insert_resource(GameMode::SingleSense)
            .insert_resource(PrintTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
            .add_systems(Startup, spawn_sensor)
            .add_systems(Update, read_sensor)
            .add_systems(Update, display_sensor);
            //.add_systems(Update, read_all_test);
    }

}
#[derive(Resource)]
struct ColorTimer(Timer);

pub struct ColorSwitcher;

impl Plugin for ColorSwitcher {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ColorTimer(Timer::from_seconds(10.0, TimerMode::Repeating)))
            .add_systems(Startup, setup_lights)
            .add_systems(Update, (turn_lights_blue, turn_lights_red).chain())
            .add_systems(Update, print_timer);
    }
}

// Define a resource to hold the LED controller
struct LedControllerResource {
    controller: Controller,
}

impl LedControllerResource {
    fn new() -> Self {
        let controller = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(
                0, // Channel Index
                ChannelBuilder::new()
                    .pin(18) // GPIO 18
                    .count(8) // Number of LEDs
                    .strip_type(StripType::Ws2812)
                    .brightness(20) // default: 255
                    .build(),
            )
            .build()
            .unwrap();
        LedControllerResource { controller }
        }
    fn set_color(&mut self, color: [u8; 4]) {
        let leds = self.controller.leds_mut(0);
        for led in leds {
            *led = color;
        }
        self.controller.render().unwrap();
    }

    fn clear_leds(&mut self) {
        self.set_color([0, 0, 0, 0]);
    }
}

fn setup_lights(world: &mut World) {
    // Initialize the LED controller and add it as a resource
    let mut led_controller = LedControllerResource::new();
    led_controller.clear_leds();
    world.insert_non_send_resource(led_controller);
}

fn turn_lights_blue(time: Res<Time>, mut timer: ResMut<ColorTimer>, mut led_controller: NonSendMut<LedControllerResource>) {
    if timer.0.tick(time.delta()).elapsed() < timer.0.tick(time.delta()).duration() / 2{
        led_controller.set_color([255, 0, 0, 0]);
    }
}

fn turn_lights_red(time: Res<Time>, mut timer: ResMut<ColorTimer>, mut led_controller: NonSendMut<LedControllerResource>) {
    if timer.0.tick(time.delta()).elapsed() >= timer.0.tick(time.delta()).duration() / 2{
        led_controller.set_color([0, 0, 255, 0]);
    }
}

fn clear_leds(mut led_controller: NonSendMut<LedControllerResource>) {
    led_controller.clear_leds();
}
fn print_timer(time: Res<Time>, mut timer: ResMut<ColorTimer>) {
    let elapsed = timer.0.tick(time.delta()).elapsed_secs();
    //erase last console printed line then print elapsed time
    print!("\x1B[1A\x1B[K");
    println!("Elapsed time: {}", elapsed);
}
fn main() {
    App::new()
        .add_plugins((MinimalPlugins, AdcPlugin))
        //.add_systems(Startup, setup_lights)
        //.add_systems(Update, clear_leds)
        .run();
}

