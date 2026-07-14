//! Complete, backend-independent easing specifications.

use std::fmt::{Debug, Formatter};
use std::num::NonZeroU16;
use std::sync::Arc;

use crate::EasingError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EaseDirection {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuiltinEase {
    Quad(EaseDirection),
    Cubic(EaseDirection),
    Quart(EaseDirection),
    Quint(EaseDirection),
    Sine(EaseDirection),
    Expo(EaseDirection),
    Circ(EaseDirection),
    Back {
        direction: EaseDirection,
        overshoot: f32,
    },
    Bounce(EaseDirection),
    Elastic {
        direction: EaseDirection,
        amplitude: f32,
        period: f32,
    },
}

impl BuiltinEase {
    pub fn sample(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            Self::Quad(direction) => directional(progress, direction, |x| x.powi(2)),
            Self::Cubic(direction) => directional(progress, direction, |x| x.powi(3)),
            Self::Quart(direction) => directional(progress, direction, |x| x.powi(4)),
            Self::Quint(direction) => directional(progress, direction, |x| x.powi(5)),
            Self::Sine(direction) => directional(progress, direction, |x| {
                1.0 - (x * std::f32::consts::FRAC_PI_2).cos()
            }),
            Self::Expo(direction) => directional(progress, direction, |x| {
                if x == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * x - 10.0)
                }
            }),
            Self::Circ(direction) => {
                directional(progress, direction, |x| 1.0 - (1.0 - x * x).max(0.0).sqrt())
            }
            Self::Back {
                direction,
                overshoot,
            } => directional(progress, direction, |x| {
                let c3 = overshoot + 1.0;
                c3 * x.powi(3) - overshoot * x.powi(2)
            }),
            Self::Bounce(direction) => directional_out(progress, direction, bounce_out),
            Self::Elastic {
                direction,
                amplitude,
                period,
            } => directional(progress, direction, |x| elastic_in(x, amplitude, period)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JumpMode {
    Start,
    End,
    None,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearPoint {
    pub input: f32,
    pub output: f32,
}

impl LinearPoint {
    pub const fn new(input: f32, output: f32) -> Self {
        Self { input, output }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringSpec {
    mass: f32,
    stiffness: f32,
    damping: f32,
    initial_velocity: f32,
    rest_speed: f32,
    rest_displacement: f32,
    natural_duration_seconds: f32,
}

impl SpringSpec {
    pub fn new(
        mass: f32,
        stiffness: f32,
        damping: f32,
        initial_velocity: f32,
        rest_speed: f32,
        rest_displacement: f32,
    ) -> Result<Self, EasingError> {
        let values = [
            mass,
            stiffness,
            damping,
            initial_velocity,
            rest_speed,
            rest_displacement,
        ];
        if values.into_iter().any(|value| !value.is_finite())
            || mass <= 0.0
            || stiffness <= 0.0
            || damping < 0.0
            || rest_speed <= 0.0
            || rest_displacement <= 0.0
        {
            return Err(EasingError::InvalidSpring);
        }
        let mut spring = Self {
            mass,
            stiffness,
            damping,
            initial_velocity,
            rest_speed,
            rest_displacement,
            natural_duration_seconds: 0.0,
        };
        spring.natural_duration_seconds = spring.compute_natural_duration_seconds();
        Ok(spring)
    }

    pub fn natural_duration_seconds(self) -> f32 {
        self.natural_duration_seconds
    }

    fn compute_natural_duration_seconds(self) -> f32 {
        const STEP_SECONDS: f32 = 1.0 / 120.0;
        const MAX_SECONDS: f32 = 60.0;
        const REQUIRED_SETTLED_SAMPLES: u8 = 8;

        let mut settled = 0_u8;
        let mut time = 0.0;
        while time <= MAX_SECONDS {
            let (position, velocity) = self.response(time);
            if (1.0 - position).abs() <= self.rest_displacement && velocity.abs() <= self.rest_speed
            {
                settled += 1;
                if settled >= REQUIRED_SETTLED_SAMPLES {
                    return time.max(STEP_SECONDS);
                }
            } else {
                settled = 0;
            }
            time += STEP_SECONDS;
        }
        MAX_SECONDS
    }

    pub fn sample(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        if progress == 0.0 || progress == 1.0 {
            return progress;
        }
        self.response(progress * self.natural_duration_seconds()).0
    }

    fn response(self, time: f32) -> (f32, f32) {
        let omega0 = (self.stiffness / self.mass).sqrt();
        let zeta = self.damping / (2.0 * (self.stiffness * self.mass).sqrt());
        let initial_displacement = 1.0;
        let initial_velocity = -self.initial_velocity;

        let (displacement, velocity) = if zeta < 1.0 - f32::EPSILON {
            let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
            let decay = (-zeta * omega0 * time).exp();
            let a = initial_displacement;
            let b = (zeta * omega0 * initial_displacement + initial_velocity) / omega_d;
            let cos = (omega_d * time).cos();
            let sin = (omega_d * time).sin();
            let displacement = decay * (a * cos + b * sin);
            let velocity = decay
                * ((-zeta * omega0) * (a * cos + b * sin)
                    + (-a * omega_d * sin + b * omega_d * cos));
            (displacement, velocity)
        } else if (zeta - 1.0).abs() <= f32::EPSILON {
            let a = initial_displacement;
            let b = initial_velocity + omega0 * initial_displacement;
            let decay = (-omega0 * time).exp();
            let displacement = (a + b * time) * decay;
            let velocity = (b - omega0 * (a + b * time)) * decay;
            (displacement, velocity)
        } else {
            let root = (zeta * zeta - 1.0).sqrt();
            let r1 = -omega0 * (zeta - root);
            let r2 = -omega0 * (zeta + root);
            let c2 = (initial_velocity - r1 * initial_displacement) / (r2 - r1);
            let c1 = initial_displacement - c2;
            let e1 = (r1 * time).exp();
            let e2 = (r2 * time).exp();
            (c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
        };
        (1.0 - displacement, -velocity)
    }
}

impl Default for SpringSpec {
    fn default() -> Self {
        let mut spring = Self {
            mass: 1.0,
            stiffness: 100.0,
            damping: 10.0,
            initial_velocity: 0.0,
            rest_speed: 0.01,
            rest_displacement: 0.005,
            natural_duration_seconds: 0.0,
        };
        spring.natural_duration_seconds = spring.compute_natural_duration_seconds();
        spring
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IrregularEase {
    pub seed: u64,
    pub strength: f32,
    pub points: NonZeroU16,
}

impl IrregularEase {
    pub fn sample(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        if progress == 0.0 || progress == 1.0 {
            return progress;
        }
        let count = self.points.get() as f32;
        let scaled = progress * count;
        let left_index = scaled.floor() as u64;
        let local = scaled.fract();
        let left = rough_point(self.seed, left_index, count, self.strength);
        let right = rough_point(self.seed, left_index + 1, count, self.strength);
        left + (right - left) * local
    }
}

pub trait EasingFunction: Send + Sync + 'static {
    fn sample(&self, progress: f32) -> f32;
}

#[derive(Clone)]
pub enum Easing {
    Linear,
    Builtin(BuiltinEase),
    CubicBezier {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Steps {
        count: NonZeroU16,
        jump: JumpMode,
    },
    LinearPoints(Arc<[LinearPoint]>),
    Irregular(IrregularEase),
    Spring(SpringSpec),
    Custom {
        name: Arc<str>,
        function: Arc<dyn EasingFunction>,
    },
}

impl Easing {
    pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Result<Self, EasingError> {
        if [x1, y1, x2, y2].into_iter().any(|value| !value.is_finite())
            || !(0.0..=1.0).contains(&x1)
            || !(0.0..=1.0).contains(&x2)
        {
            return Err(EasingError::InvalidBezierX);
        }
        Ok(Self::CubicBezier { x1, y1, x2, y2 })
    }

    pub fn linear_points(points: impl Into<Arc<[LinearPoint]>>) -> Result<Self, EasingError> {
        let points = points.into();
        if points.is_empty() {
            return Err(EasingError::EmptyLinearPoints);
        }
        if points
            .iter()
            .any(|point| !point.input.is_finite() || !point.output.is_finite())
        {
            return Err(EasingError::NonFinitePoint);
        }
        if points.windows(2).any(|pair| pair[0].input > pair[1].input) {
            return Err(EasingError::UnsortedLinearPoints);
        }
        Ok(Self::LinearPoints(points))
    }

    pub fn custom(name: impl Into<Arc<str>>, function: impl EasingFunction) -> Self {
        Self::Custom {
            name: name.into(),
            function: Arc::new(function),
        }
    }

    pub fn sample(&self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            Self::Linear => progress,
            Self::Builtin(easing) => easing.sample(progress),
            Self::CubicBezier { x1, y1, x2, y2 } => cubic_bezier(progress, *x1, *y1, *x2, *y2),
            Self::Steps { count, jump } => steps(progress, count.get(), *jump),
            Self::LinearPoints(points) => sample_linear_points(points, progress),
            Self::Irregular(easing) => easing.sample(progress),
            Self::Spring(spring) => spring.sample(progress),
            Self::Custom { function, .. } => function.sample(progress),
        }
    }

    pub fn validate(&self) -> Result<(), EasingError> {
        match self {
            Self::Linear | Self::Steps { .. } | Self::Spring(_) | Self::Custom { .. } => Ok(()),
            Self::Builtin(BuiltinEase::Back { overshoot, .. }) if !overshoot.is_finite() => {
                Err(EasingError::InvalidBuiltin)
            }
            Self::Builtin(BuiltinEase::Elastic {
                amplitude, period, ..
            }) if !amplitude.is_finite() || !period.is_finite() || *period <= 0.0 => {
                Err(EasingError::InvalidBuiltin)
            }
            Self::Builtin(_) => Ok(()),
            Self::CubicBezier { x1, y1, x2, y2 } => {
                Self::cubic_bezier(*x1, *y1, *x2, *y2).map(drop)
            }
            Self::LinearPoints(points) => Self::linear_points(Arc::clone(points)).map(drop),
            Self::Irregular(irregular)
                if !irregular.strength.is_finite() || irregular.strength < 0.0 =>
            {
                Err(EasingError::InvalidIrregular)
            }
            Self::Irregular(_) => Ok(()),
        }
    }
}

impl Default for Easing {
    fn default() -> Self {
        Self::Builtin(BuiltinEase::Cubic(EaseDirection::Out))
    }
}

impl Debug for Easing {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linear => formatter.write_str("Linear"),
            Self::Builtin(value) => formatter.debug_tuple("Builtin").field(value).finish(),
            Self::CubicBezier { x1, y1, x2, y2 } => formatter
                .debug_struct("CubicBezier")
                .field("x1", x1)
                .field("y1", y1)
                .field("x2", x2)
                .field("y2", y2)
                .finish(),
            Self::Steps { count, jump } => formatter
                .debug_struct("Steps")
                .field("count", count)
                .field("jump", jump)
                .finish(),
            Self::LinearPoints(points) => {
                formatter.debug_tuple("LinearPoints").field(points).finish()
            }
            Self::Irregular(value) => formatter.debug_tuple("Irregular").field(value).finish(),
            Self::Spring(value) => formatter.debug_tuple("Spring").field(value).finish(),
            Self::Custom { name, .. } => formatter.debug_tuple("Custom").field(name).finish(),
        }
    }
}

fn directional(progress: f32, direction: EaseDirection, ease_in: impl Fn(f32) -> f32) -> f32 {
    match direction {
        EaseDirection::In => ease_in(progress),
        EaseDirection::Out => 1.0 - ease_in(1.0 - progress),
        EaseDirection::InOut if progress < 0.5 => ease_in(progress * 2.0) / 2.0,
        EaseDirection::InOut => 1.0 - ease_in((1.0 - progress) * 2.0) / 2.0,
    }
}

fn directional_out(progress: f32, direction: EaseDirection, ease_out: impl Fn(f32) -> f32) -> f32 {
    match direction {
        EaseDirection::In => 1.0 - ease_out(1.0 - progress),
        EaseDirection::Out => ease_out(progress),
        EaseDirection::InOut if progress < 0.5 => (1.0 - ease_out(1.0 - progress * 2.0)) / 2.0,
        EaseDirection::InOut => (1.0 + ease_out(progress * 2.0 - 1.0)) / 2.0,
    }
}

fn elastic_in(progress: f32, amplitude: f32, period: f32) -> f32 {
    if progress == 0.0 || progress == 1.0 {
        return progress;
    }
    let amplitude = amplitude.abs().max(1.0);
    let period = period.abs().max(0.001);
    let phase = period / std::f32::consts::TAU * (1.0 / amplitude).asin();
    -(amplitude
        * 2.0_f32.powf(10.0 * progress - 10.0)
        * ((progress - 1.0 - phase) * std::f32::consts::TAU / period).sin())
}

fn bounce_out(mut progress: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    if progress < 1.0 / D1 {
        N1 * progress * progress
    } else if progress < 2.0 / D1 {
        progress -= 1.5 / D1;
        N1 * progress * progress + 0.75
    } else if progress < 2.5 / D1 {
        progress -= 2.25 / D1;
        N1 * progress * progress + 0.9375
    } else {
        progress -= 2.625 / D1;
        N1 * progress * progress + 0.984375
    }
}

fn cubic_bezier(progress: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    fn axis(value: f32, p1: f32, p2: f32) -> f32 {
        let inverse = 1.0 - value;
        3.0 * inverse * inverse * value * p1
            + 3.0 * inverse * value * value * p2
            + value * value * value
    }

    let mut lower = 0.0;
    let mut upper = 1.0;
    for _ in 0..16 {
        let midpoint = (lower + upper) / 2.0;
        if axis(midpoint, x1, x2) < progress {
            lower = midpoint;
        } else {
            upper = midpoint;
        }
    }
    axis((lower + upper) / 2.0, y1, y2)
}

fn steps(progress: f32, count: u16, jump: JumpMode) -> f32 {
    let count = count as f32;
    match jump {
        JumpMode::Start => ((progress * count).floor() + 1.0).min(count) / count,
        JumpMode::End => (progress * count).floor() / count,
        JumpMode::None => {
            if count <= 1.0 {
                progress
            } else {
                ((progress * count).floor() / (count - 1.0)).clamp(0.0, 1.0)
            }
        }
        JumpMode::Both => ((progress * count).floor() + 1.0) / (count + 1.0),
    }
}

fn sample_linear_points(points: &[LinearPoint], progress: f32) -> f32 {
    let first = points[0];
    if progress <= first.input {
        return first.output;
    }
    let last = points[points.len() - 1];
    if progress >= last.input {
        return last.output;
    }
    for pair in points.windows(2) {
        if progress <= pair[1].input {
            let span = pair[1].input - pair[0].input;
            if span.abs() <= f32::EPSILON {
                return pair[1].output;
            }
            let local = (progress - pair[0].input) / span;
            return pair[0].output + (pair[1].output - pair[0].output) * local;
        }
    }
    last.output
}

fn rough_point(seed: u64, index: u64, count: f32, strength: f32) -> f32 {
    let base = index as f32 / count;
    if index == 0 || index as f32 >= count {
        return base.clamp(0.0, 1.0);
    }
    let mut value = seed ^ index.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    let random = (value as f64 / u64::MAX as f64) as f32 * 2.0 - 1.0;
    (base + random * strength / count).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_eases_have_exact_endpoints() {
        let eases = [
            BuiltinEase::Quad(EaseDirection::InOut),
            BuiltinEase::Cubic(EaseDirection::Out),
            BuiltinEase::Bounce(EaseDirection::InOut),
            BuiltinEase::Elastic {
                direction: EaseDirection::Out,
                amplitude: 1.0,
                period: 0.3,
            },
        ];
        for easing in eases {
            assert_eq!(easing.sample(0.0), 0.0);
            assert_eq!(easing.sample(1.0), 1.0);
        }
    }

    #[test]
    fn linear_points_validate_order_and_sample_segments() {
        let easing = Easing::linear_points(Arc::from([
            LinearPoint::new(0.0, 0.0),
            LinearPoint::new(0.5, 0.25),
            LinearPoint::new(1.0, 1.0),
        ]))
        .unwrap();
        assert!((easing.sample(0.75) - 0.625).abs() < 0.000_1);
        assert!(Easing::linear_points(Arc::from([
            LinearPoint::new(1.0, 1.0),
            LinearPoint::new(0.0, 0.0),
        ]))
        .is_err());
    }

    #[test]
    fn spring_duration_is_finite_and_response_ends_at_one() {
        let spring = SpringSpec::default();
        assert!(spring.natural_duration_seconds().is_finite());
        assert!(spring.natural_duration_seconds() > 0.0);
        assert_eq!(spring.sample(0.0), 0.0);
        assert_eq!(spring.sample(1.0), 1.0);
    }

    #[test]
    fn irregular_easing_is_seeded() {
        let easing = IrregularEase {
            seed: 42,
            strength: 0.8,
            points: NonZeroU16::new(16).unwrap(),
        };
        assert_eq!(easing.sample(0.375), easing.sample(0.375));
        assert_ne!(
            easing.sample(0.375),
            IrregularEase { seed: 43, ..easing }.sample(0.375)
        );
    }
}
