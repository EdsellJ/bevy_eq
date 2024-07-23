

#[derive(Resource)]
struct ColorTimer(Timer);

pub struct ColorSwitcher;

impl Plugin for ColorSwitcher {
    fn build(&self, app: &mut App) {
        app
            //.insert_resource(ColorTimer(Timer::from_seconds(10.0, TimerMode::Repeating)))
            .add_systems(Startup, setup_lights);
            //.add_systems(Update, (turn_lights_blue, turn_lights_red).chain());
            //.add_systems(Update, print_timer);
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
                    .pin(LED_PIN) // GPIO 12
                    .count(NUM_RINGS*LEDS_PER_RING) // Number of LEDs
                    .strip_type(StripType::Ws2812)
                    .brightness(255) // default: 255
                    .build(),
            )
            .build()
            .unwrap();
        LedControllerResource { controller }
        }
    fn set_all_color(&mut self, color: [u8; 4]) {
        let leds = self.controller.leds_mut(0);
        for led in leds {
            *led = color;
        }
        self.controller.render().unwrap();
    }

    fn set_ring_color(&mut self, ring: i32, color: [u8; 4]) {
        let leds = self.controller.leds_mut(0);
        for i in 0..LEDS_PER_RING {
            leds[(ring * LEDS_PER_RING + i) as usize] = color;
        }
        self.controller.render().unwrap();
    }

    fn clear_leds(&mut self) {
        self.set_all_color(Colors::default().clear);
    }
}

#[derive(Component)]
// Define colors in struct form
struct Colors {
    //individual colors
    red: [u8; 4],
    green: [u8; 4],
    blue: [u8; 4],
    yellow: [u8; 4],
    purple: [u8; 4],
    white: [u8; 4],
    clear: [u8; 4],
}

impl Default for Colors {
    fn default() -> Self {
        Colors {
            red: [0, 0, 255, 0],
            green: [0, 255, 0, 0],
            blue: [255, 0, 0, 0],
            yellow: [0, 255, 255, 0],
            purple: [255, 0, 255, 0],
            white: [255, 255, 255, 0],
            clear: [0, 0, 0, 0],
        }
    }
}

// Define a component to represent an LED
#[derive(Component)]
struct Led {
    ring: i32,
    led: i32,
    color: [u8; 4],
}
fn setup_lights(world: &mut World) {
    // Initialize the LED controller and add it as a resource
    let mut led_controller = LedControllerResource::new();
    led_controller.clear_leds();
    world.insert_non_send_resource(led_controller);
}

fn turn_lights_blue(time: Res<Time>, mut timer: ResMut<ColorTimer>, mut led_controller: NonSendMut<LedControllerResource>) {
    if timer.0.tick(time.delta()).elapsed() < timer.0.tick(time.delta()).duration() / 2{
        led_controller.set_all_color(Colors::default().blue);
    }
}

fn turn_lights_red(time: Res<Time>, mut timer: ResMut<ColorTimer>, mut led_controller: NonSendMut<LedControllerResource>) {
    if timer.0.tick(time.delta()).elapsed() >= timer.0.tick(time.delta()).duration() / 2{
        led_controller.set_all_color(Colors::default().red);
    }
}

fn clear_leds(mut led_controller: NonSendMut<LedControllerResource>) {
    led_controller.clear_leds();
}