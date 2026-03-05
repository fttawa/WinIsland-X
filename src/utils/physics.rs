pub struct Spring {
    pub value: f32,
    pub velocity: f32,
}
impl Spring {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
        }
    }
    pub fn update(&mut self, target: f32, stiffness: f32, damping: f32, dt: f32) {
        // Normalize dt against 60fps (0.0166s)
        let dt_scale = (dt / 0.016666).min(4.0); 
        
        // Use sub-steps for stability and consistent feel across different frame rates
        let steps = (dt_scale.ceil() as i32).max(1);
        let step_dt = dt_scale / steps as f32;
        
        for _ in 0..steps {
            let force = (target - self.value) * stiffness;
            self.velocity = (self.velocity + force * step_dt) * damping.powf(step_dt);
            self.value += self.velocity * step_dt;
        }
    }
}
