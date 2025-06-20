use crate::{Color, ColorImage, ColorI32};
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSpace {
    RGB,
    Oklab,
}

impl Default for ColorSpace {
    fn default() -> Self {
        ColorSpace::RGB
    }
}

pub struct Runner {
    config: RunnerConfig,
    image: ColorImage,
}

pub struct RunnerConfig {
    pub diagonal: bool,
    pub hierarchical: u32,
    pub batch_size: i32,
    pub good_min_area: usize,
    pub good_max_area: usize,
    pub is_same_color_a: i32,
    pub is_same_color_b: i32,
    pub deepen_diff: i32,
    pub hollow_neighbours: usize,
    pub key_color: Color,
    pub keying_action: KeyingAction,
    pub color_space: ColorSpace,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            diagonal: false,
            hierarchical: HIERARCHICAL_MAX,
            batch_size: 25600,
            good_min_area: 16,
            good_max_area: 256 * 256,
            is_same_color_a: 4,
            is_same_color_b: 1,
            deepen_diff: 64,
            hollow_neighbours: 1,
            key_color: Color::default(),
            keying_action: KeyingAction::default(),
            color_space: ColorSpace::default(),
        }
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self {
            config: RunnerConfig::default(),
            image: ColorImage::new(),
        }
    }
}

impl Runner {

    pub fn new(config: RunnerConfig, image: ColorImage) -> Self {
        Self {
            config,
            image
        }
    }

    pub fn init(&mut self, image: ColorImage) {
        self.image = image;
    }

    pub fn builder(self) -> Builder {
        let RunnerConfig {
            diagonal,
            hierarchical,
            batch_size,
            good_min_area,
            good_max_area,
            is_same_color_a,
            is_same_color_b,
            deepen_diff,
            hollow_neighbours,
            key_color,
            keying_action,
            color_space,
        } = self.config;

        assert!(is_same_color_a < 8);

        let diff_fn = match color_space {
            ColorSpace::RGB => color_diff,
            ColorSpace::Oklab => oklab_color_diff,
        };

        Builder::new()
            .from(self.image)
            .diagonal(diagonal)
            .hierarchical(hierarchical)
            .key(key_color)
            .keying_action(keying_action)
            .batch_size(batch_size as u32)
            .same(move |a: Color, b: Color| {
                color_same(a, b, is_same_color_a, is_same_color_b)
            })
            .diff(diff_fn)
            .deepen(move |internal: &BuilderImpl, patch: &Cluster, neighbours: &[NeighbourInfo]| {
                patch_good(internal, patch, good_min_area, good_max_area) &&
                neighbours[0].diff > deepen_diff
            })
            .hollow(move |_internal: &BuilderImpl, _patch: &Cluster, neighbours: &[NeighbourInfo]| {
                neighbours.len() <= hollow_neighbours
            })
    }

    pub fn start(self) -> IncrementalBuilder {
        self.builder().start()
    }

    pub fn run(self) -> Clusters {
        self.builder().run()
    }

}

pub fn color_diff(a: Color, b: Color) -> i32 {
    let a = ColorI32::new(&a);
    let b = ColorI32::new(&b);
    (a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs()
}

pub fn oklab_color_diff(a: Color, b: Color) -> i32 {
    let a_oklab: oklab::Oklab = oklab::Rgb { r: a.r, g: a.g, b: a.b }.into();
    let b_oklab: oklab::Oklab = oklab::Rgb { r: b.r, g: b.g, b: b.b }.into();

    let dl = a_oklab.l - b_oklab.l;
    let da = a_oklab.a - b_oklab.a;
    let db = a_oklab.b - b_oklab.b;
    
    // Squared Euclidean distance
    let delta_sq = dl * dl + da * da + db * db;

    // The Oklab values are floats. `l` is in [0, 1], `a` and `b` are roughly in [-0.4, 0.4].
    // The squared difference will be small.
    // Let's scale it up to be in a similar range to color_diff.
    // A simple scaling factor should work for comparison purposes.
    (delta_sq * 255.0) as i32
}

pub fn color_same(a: Color, b: Color, shift: i32, thres: i32) -> bool {
    let diff = ColorI32 {
        r: (a.r >> shift) as i32,
        g: (a.g >> shift) as i32,
        b: (a.b >> shift) as i32,
    }
    .diff(&ColorI32 {
        r: (b.r >> shift) as i32,
        g: (b.g >> shift) as i32,
        b: (b.b >> shift) as i32,
    });

    diff.r.abs() <= thres && diff.g.abs() <= thres && diff.b.abs() <= thres
}

fn patch_good(
    internal: &BuilderImpl,
    patch: &Cluster,
    good_min_area: usize,
    good_max_area: usize
) -> bool {
    if good_min_area < patch.area() && patch.area() < good_max_area {
        if good_min_area == 0 ||
            (patch.perimeter_internal(internal) as usize) < patch.area() {
            return true;
        } else {
            // cluster is thread-like and thinner than 2px
        }
    }
    false
}
