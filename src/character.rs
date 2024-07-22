use godot::{
    classes::ICharacterBody2D,
    engine::{CharacterBody2D, CollisionShape2D, Engine, RectangleShape2D},
    prelude::*,
};

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum Kind {
    #[default]
    Sonic,
    Tails,
    Knuckles,
}

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum State {
    #[default]
    Standing,
    Ball,
}

#[derive(GodotClass)]
#[class(tool,init, base=CharacterBody2D)]
struct Character {
    #[export]
    #[var(get, set = set_character)]
    character: Kind,
    #[export]
    #[var(get, set = set_state)]
    state: State,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_width_radius)]
    #[init(default = 19.0)]
    width_radius: f32,
    #[export(range = (0.0,100.0, 1.0))]
    #[var(get, set = set_height_radius)]
    #[init(default = 39.0)]
    height_radius: f32,
    #[export]
    #[init(default = 20.0)]
    push_radius: f32,
    #[export]
    #[init(default = 6.5)]
    jump_force: f32,
    #[export]
    collision_shape: Option<Gd<CollisionShape2D>>,

    base: Base<CharacterBody2D>,
}

#[godot_api]
impl ICharacterBody2D for Character {
    fn physics_process(&mut self, delta: f64) {
        if Engine::singleton().is_editor_hint() {
            return;
        }
        self.base_mut().set_velocity(Vector2::new(0.0, 980.0));
        self.base_mut().move_and_slide();
    }
}

#[godot_api]
impl Character {
    #[func]
    fn set_width_radius(&mut self, value: f32) {
        self.width_radius = value;
        let Some(mut rectangle) = self.get_rectangle_mut() else {
            return;
        };

        let y = rectangle.get_size().y;
        rectangle.set_size(Vector2::new(self.width_radius, y));
    }

    #[func]
    fn set_height_radius(&mut self, value: f32) {
        self.height_radius = value;
        let Some(mut rectangle) = self.get_rectangle_mut() else {
            return;
        };

        let x = rectangle.get_size().x;
        rectangle.set_size(Vector2::new(x, self.height_radius));

        self.set_shape_y(-self.height_radius / 2.0);
    }

    #[func]
    fn set_character(&mut self, value: Kind) {
        match value {
            Kind::Sonic => {
                self.set_width_radius(19.0);
                self.set_height_radius(39.0);
                self.jump_force = 6.5;
            }
            Kind::Tails => {
                self.set_width_radius(19.0);
                self.set_height_radius(31.0);
                self.jump_force = 6.5;
            }
            Kind::Knuckles => {
                self.set_width_radius(19.0);
                self.set_height_radius(39.0);
                self.jump_force = 6.0;
            }
        }

        self.character = value;
    }
    #[func]
    fn set_state(&mut self, value: State) {
        match (self.state, value) {
            (State::Standing, State::Ball) => {
                self.set_height_radius(20.0);
            }
            (State::Ball, State::Standing) => {
                self.set_character(self.character);
            }
            _ => {}
        }
        self.state = value;
    }

    fn set_shape_y(&mut self, amount: f32) {
        if let Some(collision_shape) = self.collision_shape.as_deref_mut() {
            let mut position = collision_shape.get_position();
            position.y = amount;
            collision_shape.set_position(position)
        }
    }

    fn get_rectangle(&self) -> Option<Gd<RectangleShape2D>> {
        self.collision_shape
            .as_deref()?
            .get_shape()?
            .try_cast()
            .ok()
    }
    fn get_rectangle_mut(&mut self) -> Option<Gd<RectangleShape2D>> {
        self.collision_shape
            .as_deref_mut()?
            .get_shape()?
            .try_cast()
            .ok()
    }
}
