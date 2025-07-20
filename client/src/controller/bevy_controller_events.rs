use bevy::{ecs::event::Event, math::Vec3};

#[derive(Event, Debug)]
pub struct PositionChangeEvent {
    pub prev_pos: Vec3,
    pub new_pos: Vec3,
}

#[derive(Debug)]
pub enum ActionType {
    RightClick,
    LeftClick,
}

#[derive(Event, Debug)]
pub struct ActionEvent {
    pub action_type: ActionType,
    pub controller_forward: Vec3,
    pub controller_pos: Vec3,
}
