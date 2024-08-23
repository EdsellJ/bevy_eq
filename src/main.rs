use rand::Rng;
use bevy::prelude::*;
use crate::led_controller::*;
use crate::sensor_controller::*;
use rust_gpiozero::Button;
use std::time::Duration;

pub mod led_controller;
pub mod sensor_controller;


#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameMode {
    Warmup,
    FullBody,
    Eyes,
    Combo1,
    Combo2,
    Combo3,
    Menu
}
pub struct Menu;

impl Plugin for Menu {
    fn build(&self, app: &mut App) {
        app
            
    }
}

//system that check if a button is pressed
fn button_pressed(){
    // Create a button which is attached to Pin 17
    let mut button = Button::new(4);
    // Add debouncing so that subsequent presses within 100ms don't trigger a press
    //.debounce(Duration::from_millis(100));

button.wait_for_press(None);
println!("button pressed");
}
pub struct Warmup;

impl Plugin for Warmup {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((MinimalPlugins, ColorSwitcher))
            .insert_resource(Mcp3208Resource::new())
            //.add_systems(Startup, ((spawn_random_id, print_id), spawn_sensor.chain()))
            .add_systems(Update, read_specific_sensor).run_if()
            .add_systems(Update, display_sensor)
            .add_systems(Update, full_body_update)            
            .add_systems(Update, ((update_random_id, print_id), (spawn_sensor, light_to_hit_green).chain()).run_if(check_if_none_exist));
    }   
}

fn light_to_hit_green(mut led_controller: NonSendMut<LedControllerResource>, sensor: Query<(&Sensor, Entity)>){
    for (sensor, _) in sensor.iter(){
        led_controller.set_ring_color(sensor.id as i32, Colors::default().green);
    }
}
fn check_if_none_exist(q_sensor: Query<&Sensor>) -> bool{
    //check if a sensor exists
    for sensor in q_sensor.iter(){
        if sensor.id >= 0{ 
            return false;
        }
    }
    true
}


fn print_id(query_id: Query<&ID>){
    for id in query_id.iter(){
        println!("ID: {}", id.id);
    }
}

fn spawn_random_id(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let val = rng.gen_range(0..26);
    commands.spawn(ID { id: val });
}

fn update_random_id(mut query_id: Query<&mut ID>){
    let mut the_id = query_id.single_mut();
        let mut rng = rand::thread_rng();
        let val = rng.gen_range(0..26);
        the_id.id = val;
    
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

pub struct Eyes;

impl Plugin for Eyes {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((MinimalPlugins, ColorSwitcher))
            .insert_resource(Mcp3208Resource::new())
            .add_systems(Startup, (spawn_eyes_id, spawn_sensor).chain())
            .add_systems(Update, read_specific_sensor)
            .add_systems(Update, display_sensor)
            .add_systems(Update, glowing_eyes);
    }
}
fn spawn_eyes_id(mut commands: Commands) {
    //spawn id 3 and 6
    commands.spawn(ID { id: 2 });
    commands.spawn(ID { id: 5 });
}
fn glowing_eyes(mut led_controller: NonSendMut<LedControllerResource>, sensor: Query<(&Sensor, Entity)>) {
    //loop led 3 and 6 to change color from orange to red to purple and back to orange smoothly
    /*
        red: [0, 0, 255, 0],
        green: [0, 255, 0, 0],
        blue: [255, 0, 0, 0],
        yellow: [0, 255, 255, 0],
        purple: [255, 0, 255, 0],
        orange: [255, 255, 165, 0],
        white: [255, 255, 255, 0],
    */
    let mut r = 50;
    let mut g = 255;
    let mut b = 0;
    let mut up = true;
    loop{
        for (sensor, _) in sensor.iter(){
            //set orange to start
            led_controller.set_ring_color(sensor.id as i32, [b, r, g, 0]);
        }
        if up{
            if r < 255{
                r += 5;
            }
            else if g > 0{
                g -= 5;
            }
            else if b < 255{
                b += 5;
            }
            else{
                up = false;
            }
        }
        else{
            if r > 0{
                r -= 5;
            }
            else if g < 255{
                g += 5;
            }
            else if b > 0{
                b -= 5;
            }
            else{
                up = true;
            }
        }
    }

}

#[derive(Resource)]
pub struct Combinations {
    combo1: [u8; 5],
    combo2: [u8; 3],
    combo3: [u8; 4],
    current_combo: u8,
    current_index: u8,
}
impl Combinations {
    fn new() -> Self {
        let combo1: [u8; 5] = [23, 12, 3, 1, 6];
        let combo2: [u8; 3]=  [12, 3, 7];
        let combo3:[u8; 4] = [9, 12, 0, 6];
        let current_combo: u8 = 1;
        let current_index: u8 = 0;
        
        Combinations {
            combo1,
            combo2,
            combo3,
            current_combo,
            current_index,
        }
    }
}

pub struct Combo1;
impl Plugin for Combo1{
    fn build(&self, app: &mut App){
        app
            .add_plugins((MinimalPlugins, ColorSwitcher))
            .insert_resource(Mcp3208Resource::new())
            .insert_resource(Combinations::new())
            .add_systems(Startup, (combo_1_startup, spawn_sensor).chain())
            .add_systems(Update, read_specific_sensor)
            .add_systems(Update, display_sensor)
            .add_systems(Update, full_body_update)           
            .add_systems(Update, light_to_hit_green) 
            .add_systems(Update, (combo_1_update, spawn_sensor, read_sensor, display_sensor).chain().run_if(check_if_none_exist));
    }
}
fn combo_1_update(mut commands: Commands, mut combo: ResMut<Combinations>, mut query_id: Query<&mut ID>){
    //get index from combo struct
    let index = combo.current_index;
    let ids = combo.combo1;
            
    let mut the_id = query_id.single_mut();
    the_id.id = ids[index as usize];

    //increment the index if it is not the last index
    if index < 4{
        combo.current_index += 1;
    }
    else{
        combo.current_index = 0;
    }
}
fn combo_1_startup(mut commands: Commands, mut combo: ResMut<Combinations>, query_id: Query<&ID>){
    //combo 1 is 24, 13, 4, 2, 7
    
    //get index from combo struct
    let index = combo.current_index;
    let ids = combo.combo1;
            
    commands.spawn(ID { id: ids[index as usize]});
    //increment the index
    combo.current_index += 1;
}

pub struct Combo2;
impl Plugin for Combo2{
    fn build(&self, app: &mut App){
        app
            .add_plugins((MinimalPlugins, ColorSwitcher))
            .insert_resource(Mcp3208Resource::new())
            .insert_resource(Combinations::new())
            .add_systems(Startup, (combo_2_startup, spawn_sensor).chain())
            .add_systems(Update, read_specific_sensor)
            .add_systems(Update, display_sensor)
            .add_systems(Update, full_body_update)           
            .add_systems(Update, light_to_hit_green) 
            .add_systems(Update, (combo_2_update, spawn_sensor, read_sensor, display_sensor).chain().run_if(check_if_none_exist));
    }
}
fn combo_2_update(mut commands: Commands, mut combo: ResMut<Combinations>, mut query_id: Query<&mut ID>){
    //get index from combo struct
    let index = combo.current_index;
    let ids = combo.combo2;
            
    let mut the_id = query_id.single_mut();
    the_id.id = ids[index as usize];

    //increment the index if it is not the last index
    if index < 2{
        combo.current_index += 1;
    }
    else{
        combo.current_index = 0;
    }
}
fn combo_2_startup(mut commands: Commands, mut combo: ResMut<Combinations>, query_id: Query<&ID>){
    //combo 1 is 24, 13, 4, 2, 7
    
    //get index from combo struct
    let index = combo.current_index;
    let ids = combo.combo2;
            
    commands.spawn(ID { id: ids[index as usize]});
    //increment the index
    combo.current_index += 1;
}

pub struct Combo3;
impl Plugin for Combo3{
    fn build(&self, app: &mut App){
        app
            .add_plugins((MinimalPlugins, ColorSwitcher))
            .insert_resource(Mcp3208Resource::new())
            .insert_resource(Combinations::new())
            .add_systems(Startup, (combo_3_startup, spawn_sensor, combo_3_update, spawn_sensor).chain())
            .add_systems(Update, read_specific_sensor)
            .add_systems(Update, display_sensor)
            .add_systems(Update, full_body_update)           
            .add_systems(Update, light_to_hit_green) 
            //.add_systems(Update, (combo_3_update, spawn_sensor, combo_3_update, spawn_sensor, read_sensor, display_sensor).chain().run_if(check_if_none_exist).run_if(check_if_at_index_0))
            .add_systems(Update, (combo_3_update, spawn_sensor, read_sensor, display_sensor).chain().run_if(check_if_none_exist));
    }
}
fn combo_3_update(mut commands: Commands, mut combo: ResMut<Combinations>, mut query_id: Query<&mut ID>){
    //get index from combo struct
    let index = combo.current_index;
    let ids = combo.combo3;
            
    let mut the_id = query_id.single_mut();
    the_id.id = ids[index as usize];

    //increment the index if it is not the last index
    if index < 3{
        combo.current_index += 1;
    }
    else{
        combo.current_index = 0;
    }
}
fn combo_3_startup(mut commands: Commands, mut combo: ResMut<Combinations>, query_id: Query<&ID>){
    //combo 1 is 24, 13, 4, 2, 7
    
    //get index from combo struct
    let index = combo.current_index;
    let ids = combo.combo3;
            
    commands.spawn(ID { id: ids[index as usize]});
    //increment the index
    combo.current_index += 1;
}

fn check_if_at_index_0(q_combo: Res<Combinations>) -> bool{
    if q_combo.current_index == 0{
        return true;
    }
    false
}

fn check_if_not_at_index_0(q_combo: Res<Combinations>) -> bool{
    if q_combo.current_index == 0{
        return false;
    }
    true
}

pub struct FullBody;

impl Plugin for FullBody {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((MinimalPlugins, AdcPlugin, ColorSwitcher))
            //.add_systems(Startup, spawn_all_sensors)
            .add_systems(Startup, set_all_green)
            .add_systems(Update, full_body_update);
    }
}
fn set_all_green(sensor: Query<(&Sensor, Entity)>, mut led_controller: NonSendMut<LedControllerResource>) {
    led_controller.set_all_color(Colors::default().green);
}

fn full_body_update(mut commands: Commands, sensor: Query<(&Sensor, Entity)>, mut led_controller: NonSendMut<LedControllerResource>) {
    for (sensor, ent) in sensor.iter(){
        let value = sensor.value;
        match value {
            250..=600 => {
                led_controller.set_ring_color(sensor.id as i32, Colors::default().red);
                std::thread::sleep(std::time::Duration::from_millis(20));
                led_controller.set_ring_color(sensor.id as i32, Colors::default().clear);
                //despawn sensor
                commands.entity(ent).despawn();
            }
            601..=2000 => {
                led_controller.set_ring_color(sensor.id as i32, Colors::default().purple);
                //delay for 50ms
                std::thread::sleep(std::time::Duration::from_millis(200));
                led_controller.set_ring_color(sensor.id as i32, Colors::default().clear);
                commands.entity(ent).despawn();

            }
            _ => {
                continue;
            }
        }
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
        .add_systems(Update, button_pressed)
        //.add_plugins(Combo3)
        //.add_plugins(Eyes)
        //.add_plugins(FullBody)
        //.add_plugins(Warmup)
        .run();
}
        
        
        //
        //.add_systems(Update, standby)