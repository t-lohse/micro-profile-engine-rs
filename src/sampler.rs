use crate::profile::{ControlType, InterpolationType, Point};
use crate::profile::{Flow, Percent, Pressure, YVal};
use crate::profile::{InputType, Stage};

#[derive(Debug)]
struct SamplerPoint {
    pub x: f64,
    pub y: f64,
}

impl SamplerPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn from_ctr_point(ctrl: ControlType, point: Point, unit_conversion: i64) -> Self {
        //let y = match ctrl {};
        println!("ctrl: {ctrl:?} - {:?}", point.y);
        let y = point.y.value().into();
        let x = point.x as f64 / 10f64 * unit_conversion as f64;
        Self { x, y }
    }
}

pub struct Sampler {
    points: Vec<SamplerPoint>,
    interpolation: InterpolationType,
    time_series_index: usize,
    current_segment_start: Option<f64>,
    current_segment_end: Option<f64>,
    pub stage_id: u16,
}

impl Sampler {
    pub fn get(&mut self, current_ref_input: f64) -> f64 {
        println!("{current_ref_input} -- {:?}", self.points);
        match <Vec<_> as AsRef<[_]>>::as_ref(&self.points) {
            [one] => one.y,
            [first, ..] if first.x >= current_ref_input => first.y,
            [.., last] if last.x <= current_ref_input => last.y,
            _ => self.get_value_linear(current_ref_input), // TODO: Should be linear
        }
        //let first_point = self.points.first()
    }

    pub fn load_from_stage(stage: &Stage, stage_id: u16) -> Self {
        let (ctrl_type, interpolation, conversion_factor, points) = {
            let _dyn = stage.dynamics();
            let i = if _dyn.input_type() == InputType::InputTime {
                1000
            } else {
                1
            };

            (_dyn.ctrl(), _dyn.interpolation(), i, _dyn.points())
        };

        for p in points {}

        let points = Vec::from_iter(points.into_iter().map(|p| {
            println!(
                "InputPoint: ({}:{})",
                p.x as f64 / 10.0f64,
                p.y.value() as f64 / 10.0f64
            );
            SamplerPoint::from_ctr_point(ctrl_type, *p, conversion_factor)
        }));

        println!(
            "Loading stage {} with {} points into the sampler",
            stage_id,
            points.len()
        );

        let interpolation = interpolation;
        let time_series_index = 0;
        let stage_id = stage_id;

        for p in &points {
            println!("SamplerPoint: ({}:{})", p.x, p.y)
        }

        Self {
            points,
            interpolation,
            time_series_index,
            stage_id,
            current_segment_start: None,
            current_segment_end: None,
        }
    }

    pub fn load_new_stage(&mut self, stage: &Stage, stage_id: u16) {
        *self = Self::load_from_stage(stage, stage_id);
    }

    fn find_current_segment(&mut self, current_value: f64) {
        self.current_segment_end = self.points.iter().enumerate().find_map(|(i, p)| {
            if current_value > p.x {
                self.time_series_index = i;
                Some(p.x)
            } else {
                self.current_segment_start = Some(p.x);
                None
            }
        });

        if self.current_segment_end.is_none() {
            return;
        }

        self.current_segment_start = self.points.iter().enumerate().find_map(|(i, p)| {
            if current_value < p.x {
                self.time_series_index = i;
                Some(p.x)
            } else {
                self.current_segment_end = Some(p.x);
                None
            }
        });
    }

    fn get_value_linear(&mut self, current_value: f64) -> f64 {
        self.find_current_segment(current_value);
        match (self.current_segment_start, self.current_segment_end) {
            (None, _) => return self.points[0].y,
            (_, None) => return self.points.last().unwrap().y,
            _ => (),
        };

        let slope = (self.points[self.time_series_index].y
            - self.points[self.time_series_index - 1].y)
            / (self.points[self.time_series_index].x - self.points[self.time_series_index - 1].x);

        let intercept =
            self.points[self.time_series_index].y - slope * (self.points[self.time_series_index].x);
        (slope * current_value) + intercept
    }
}

impl Default for Sampler {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            interpolation: InterpolationType::default(),
            time_series_index: 0,
            current_segment_start: None,
            current_segment_end: None,
            stage_id: u16::MAX,
        }
    }
}

//impl From<Point> for SamplerPoint
