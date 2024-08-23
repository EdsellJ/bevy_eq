use rand::Rng;
use bevy::prelude::*;
use crate::led_controller::*;
use crate::sensor_controller::*;
use rust_gpiozero::Button;
use std::time::Duration;
//use threading
use std::thread;

pub mod led_controller;
pub mod sensor_controller;


#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameMode {
    Warmup,
    Eyes,
    FullBodyStartup,
    FullBodyRun,
    Combo1,
    Combo2,
    Combo3,
    Menu
}
pub struct Menu;

impl Plugin for Menu {
    fn build(&self, app: &mut App) {
        app
            //mode switching systems
            .add_systems(Update,((mode_switcher_id_select, spawn_sensor, light_to_hit_green, mode_switcher).chain()).run_if(in_state(GameMode::Menu)));
    }
}


fn mode_switcher_id_select(mut commands: Commands, mut query_id: Query<&mut ID>) {
    //spawn sensors 2, 3, 4, 5, 6, 7
    //if ids 2-8 do not exist then spawn
    for i in 1..7 {
        let mut exists = false;
        for id in query_id.iter() {
            if id.id == i {
                exists = true;
                break;  // Exit early if the ID is found
            }
        }
        if !exists {
            commands.spawn(ID { id: i });
        }
    }
}

fn mode_switcher(state: Res<State<GameMode>>, mut next_state: ResMut<NextState<GameMode>>, sensor: Query<(&Sensor, Entity)>) {
    //switch to the next state
    for (sensor, _) in sensor.iter(){
        let value = sensor.value;
        match value {
            100..=2000 => {
                match sensor.id{
                    1 => {
                        next_state.set(GameMode::Warmup);
                    }
                    2 => {
                        next_state.set(GameMode::Eyes);
                    }
                    3 => {
                        next_state.set(GameMode::FullBodyStartup);
                    }
                    4 => {
                        next_state.set(GameMode::Combo1);
                    }
                    5 => {
                        next_state.set(GameMode::Combo2);
                    }
                    6 => {
                        next_state.set(GameMode::Combo3);
                    }
                    _ => {
                        continue;
                    }
                }
            }
            _ => {
                continue;
            }
        }
    }
}

fn button_pressed(mut next_state: ResMut<NextState<GameMode>>) {
     // Create a button which is attached to Pin 17
     let button = Button::new(4);
     // Add debouncing so that subsequent presses within 100ms don't trigger a press
     //.debounce(Duration::from_millis(100));
     // timeout for 1ms
    if button.value() == true{
        next_state.set(GameMode::Menu)
    }
     //change state to Menu
}

pub struct Warmup;

impl Plugin for Warmup {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, full_body_update.run_if(in_state(GameMode::Warmup)))
            .add_systems(Update, ((despawn_sensors, spawn_random_id, print_id, spawn_sensor, light_to_hit_green).chain())
                .run_if(check_if_none_exist)
                .run_if(in_state(GameMode::Warmup)));

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
            .add_systems(Startup, spawn_all_sensors)
            .add_systems(Update, read_sensor)
            .add_systems(Update, display_sensor);
    }

}

pub struct Eyes;

impl Plugin for Eyes {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (spawn_eyes_id, spawn_sensor).chain().run_if(in_state(GameMode::Eyes)))
            .add_systems(Update, glowing_eyes.run_if(in_state(GameMode::Eyes)));
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
    for i in 0..250{
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
            .add_systems(Update, full_body_update.run_if(in_state(GameMode::Combo1)))
            .add_systems(Update, (despawn_sensors, combo_1_startup, spawn_sensor, light_to_hit_green).chain().run_if(check_if_none_exist).run_if(in_state(GameMode::Combo1)));
    }
}
fn combo_1_startup(mut commands: Commands, mut combo: ResMut<Combinations>, query_id: Query<&ID>){
    //combo 1 is 24, 13, 4, 2, 7
    
    //get index from combo struct
    let index = combo.current_index;
    let ids = combo.combo1;
            
    commands.spawn(ID { id: ids[index as usize]});
    //increment the index if it is not the last index
    if index < 4{
        combo.current_index += 1;
    }
    else{
        combo.current_index = 0;
    }
}

pub struct Combo2;
impl Plugin for Combo2{
    fn build(&self, app: &mut App){
        app   
            .add_systems(Update, full_body_update.run_if(in_state(GameMode::Combo2)))
            .add_systems(Update, (despawn_sensors, combo_2_startup, spawn_sensor, light_to_hit_green).chain().run_if(check_if_none_exist).run_if(in_state(GameMode::Combo2)));
    }
}

fn combo_2_startup(mut commands: Commands, mut combo: ResMut<Combinations>, query_id: Query<&ID>){
    //combo 1 is 24, 13, 4, 2, 7
    
    //get index from combo struct
    let index = combo.current_index;
    let ids = combo.combo2;
            
    commands.spawn(ID { id: ids[index as usize]});
    //increment the index
    if index < 2{
        combo.current_index += 1;
    }
    else{
        combo.current_index = 0;
    }
}

pub struct Combo3;
impl Plugin for Combo3{
    fn build(&self, app: &mut App){
        app   
            .add_systems(Update, full_body_update.run_if(in_state(GameMode::Combo3)))
            .add_systems(Update, (despawn_sensors, combo_3_startup, spawn_sensor, light_to_hit_green).chain().run_if(check_if_none_exist).run_if(in_state(GameMode::Combo3)));
    }
}

fn combo_3_startup(mut commands: Commands, mut combo: ResMut<Combinations>, query_id: Query<&ID>){
    //combo 1 is 24, 13, 4, 2, 7
    
    //get index from combo struct
    let index = combo.current_index;
    let ids = combo.combo3;
            
    commands.spawn(ID { id: ids[index as usize]});
    //increment the index
    if index < 3{
        combo.current_index += 1;
    }
    else{
        combo.current_index = 0;
    }
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
            .add_systems(Update, (spawn_all_sensors, light_to_hit_green, full_body_transition).chain().run_if(in_state(GameMode::FullBodyStartup)))
            .add_systems(Update, full_body_update.run_if(in_state(GameMode::FullBodyRun)));
    }
}
fn full_body_transition(mut next_state: ResMut<NextState<GameMode>>) {
    //switch to the next state
    next_state.set(GameMode::FullBodyRun);
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

pub struct StateTransitionHandler;

impl Plugin for StateTransitionHandler {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameMode::Menu),(
                despawn_sensors,
                clear_leds
            ))
            .add_systems(OnExit(GameMode::Menu),(
                despawn_sensors,
                clear_leds
            ));
    }
}
fn main() {
    App::new()
        .insert_state(GameMode::Menu)
        .add_plugins((MinimalPlugins, ColorSwitcher, StateTransitionHandler))
        .insert_resource(Mcp3208Resource::new())
        .insert_resource(Combinations::new())
        .add_plugins(Menu)
        .add_systems(Update, button_pressed)
        .add_systems(Update, (read_sensor, display_sensor))
        .add_plugins(Combo1)
        .add_plugins(Combo2)
        .add_plugins(Combo3)
        .add_plugins(Eyes)
        .add_plugins(FullBody)
        .add_plugins(Warmup)
        .run();


}
        
        
        //
        //.add_systems(Update, standby)