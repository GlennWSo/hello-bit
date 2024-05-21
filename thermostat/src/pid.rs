use core::cmp::max;

pub struct PID {
    //cfg
    pk: f32,
    ik: f32,
    dk: f32,
    min_output: f32,
    max_output: f32,
    //state
    target: f32,
    sum_diff: f32,
    old_feedback: f32,
}

impl PID {
    pub fn new(pk: f32, ik: f32, dk: f32, min_output: f32, max_output: f32, target: f32) -> Self {
        assert!(max_output > min_output);
        assert!(dk <= 0.);
        Self {
            pk,
            ik,
            dk,
            min_output,
            max_output,
            target,
            sum_diff: 0.,
            old_feedback: 0.,
        }
    }
    pub fn update(&mut self, feedback: f32, dt: f32) -> f32 {
        let diff = self.target - feedback;
        let p = self.pk * diff;

        self.sum_diff += diff;
        let i = self.sum_diff * self.ik;

        let v = (feedback - self.old_feedback) / dt;
        self.old_feedback = feedback;
        let d = v * self.dk;
        let output = (p + i + d).max(self.min_output).min(self.max_output);
        output
    }
}
