use godot::{
    engine::{Area2D, IArea2D},
    prelude::*,
};

#[derive(GodotConvert, Var, Export, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[godot(via = GString)]
enum Type {
    #[default]
    Attackable,
    Increment,
    Hurt,
    Special,
    Player,
}

#[derive(GodotClass)]
#[class(init, base=Area2D)]
struct Hitbox {
    #[export]
    hitbox_type: Type,
    base: Base<Area2D>,
}
#[godot_api]
impl IArea2D for Hitbox {
    fn ready(&mut self) {
        if self.hitbox_type != Type::Player {
            let callable = self.base().callable("on_area_entered");
            self.base_mut().connect(c"area_entered".into(), callable);
        }
    }
}
#[godot_api]
impl Hitbox {
    #[func]
    fn on_area_entered(&mut self, area: Gd<Area2D>) {
        if let Ok(hitbox) = area.try_cast::<Hitbox>() {
            if hitbox.bind().hitbox_type == Type::Player {
                match self.hitbox_type {
                    Type::Attackable => {}
                    Type::Increment => {}
                    Type::Hurt => {}
                    Type::Special => {}
                    Type::Player => {}
                }
            }
        }
    }
}

impl Hitbox {
    fn event_bus_singleton(&self) -> Option<Gd<Node>> {
        self.base().get_node_or_null("/root/EventBus".into())
    }
}
