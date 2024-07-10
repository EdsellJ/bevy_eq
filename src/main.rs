use bevy::prelude::*;
use bevy::time::Fixed;
use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder, StripType};
use mcp3208::{Channel, Mcp3208};

#[derive(Resource)]
struct Mcp3208Resource{
    mcp3208: Mcp3208,
    //array of Channel enum
    channel: [Channel; 8]
}

impl Mcp3208Resource{
    fn new() -> Self{
        let mcp3208 = Mcp3208::new("/dev/spidev0.0").unwrap();
        let channel: [Channel; 8] = [Channel::Ch0, Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4, Channel::Ch5, Channel::Ch6, Channel::Ch7];
        Mcp3208Resource{mcp3208, channel}
    }
    fn read_channel(&mut self, chnl: Channel) -> u16{
        self.mcp3208.read_adc_diff(chnl).unwrap()
    }
}
 
//create a component for channel
#[derive(Component)]
struct Sensor{
    channel: u8
}

fn read_adc(mut mcp3208: ResMut<Mcp3208Resource>){
    let channel = mcp3208.channel;
    //read adc value from channel 0
    let value = mcp3208.read_channel(channel[0]);
    //erase last console printed line then print adc value
    print!("\x1B[1A\x1B[K");
    println!("ADC Value: {}", value);
    //sleep for 1 ms
    std::thread::sleep(std::time::Duration::from_millis(5));
}
pub struct adcPlugin;

impl Plugin for adcPlugin{
    fn build(&self, app: &mut App){
        app.insert_resource(Mcp3208Resource::new())
            .add_systems(Update, read_adc);
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
        .add_plugins((MinimalPlugins, adcPlugin))
        //.add_systems(Startup, setup_lights)
        //.add_systems(Update, clear_leds)
        .run();
}

