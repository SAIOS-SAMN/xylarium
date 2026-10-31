// The poles as conformal points on the pole axis: Z at the origin e_o, Q at
// the point at infinity, a carrier's state at t = U / W (open frames over
// closed) and the throat at t = 1.
//
// Inputs: `xyl_core::cga` (points, dilators, projective comparison).
//
// Outputs: `pole_point` (X(t)) and `pole_flux`, which reads an echo's
// observed dilation t1 / t0, composes each booked channel as a dilator on
// the state it finds, and returns a `PoleFlux` with each channel's
// dilation, the residual (observed over the product) and whether the
// composed versor equals the observed one projectively. Exact; None where a
// ratio is undefined.
// Terms: filament (docs/glossary.md).

use xyl_core::cga::{self, Mv};
use xyl_core::Q;

/// The poles as conformal points, built with exact versors. The Z pole
/// (holding, coherence) is the origin e_o and the Q pole (magnetism,
/// decoherence) the point at infinity e_inf; a carrier's pole state is the
/// point X(t) on the pole axis at t = U / W (open frames over closed). The
/// vacuum's inversion, with the throat as its one fixed point, is the unit
/// sphere's inversion X(t) -> X(1 / t): it swaps the poles and fixes t = 1.
pub fn pole_point(w: &Q, u: &Q) -> Option<Mv> {
    let t = u.checked_div(w)?;
    Some(cga::point_at(&[t]))
}

/// The flux as a versor: an echo carries X(t0) to X(t1) by the dilator D(rho),
/// rho = t1 / t0 - toward Z (the origin) where rho < 1, toward Q (infinity)
/// where rho > 1. Each booked channel moving `toward_z` frames from Q to Z
/// (negative: Z to Q) is its own dilator on the state it finds; channels
/// compose by the geometric product, and the books close where the composed
/// versor is projectively equal to the observed one. Any unbooked channel
/// remains as the residual dilator.
pub struct PoleFlux {
    /// The pole point's coordinate before and after (t = U / W).
    pub t0: Q,
    /// The pole point's coordinate after the echo, U1 / W1.
    pub t1: Q,
    /// The observed dilation, each channel's, and the residual (observed over
    /// the channels' product: 1 where the books close).
    pub rho: Q,
    /// Each booked channel's name and its own dilation, in booking order.
    pub channels: Vec<(&'static str, Q)>,
    /// The observed dilation over the channels' product: 1 where the books close.
    pub residual: Q,
    /// Whether the composed channel versor is projectively the observed one.
    pub closed: bool,
}

/// Reads an echo's pole flux from the closed frames `w0`, `w1` and open frames
/// `u0`, `u1` before and after, composing each `booked` channel (name, frames moved
/// from Q to Z) as a dilator on the state it finds.
///
/// Returns `None` when a ratio along the way is undefined: a zero closed-frame
/// count, a zero starting coordinate U0 / W0, or a channel that empties the
/// open frames. Read per firmament kind and echo by `Substrate::loop_flux`,
/// the jetstream.
pub fn pole_flux(w0: &Q, u0: &Q, w1: &Q, u1: &Q, booked: &[(&'static str, Q)]) -> Option<PoleFlux> {
    let (t0, t1) = (u0.checked_div(w0)?, u1.checked_div(w1)?);
    let rho = t1.checked_div(&t0)?;
    let observed = cga::dilator(&rho);
    let (mut w, mut u) = (w0.clone(), u0.clone());
    let mut composed = Mv::scalar(Q::ONE);
    let mut channels = Vec::with_capacity(booked.len());
    let mut product = Q::ONE;
    for (name, z) in booked {
        let t_before = u.checked_div(&w)?;
        w = &w + z;
        u = &u - z;
        let rho_c = u.checked_div(&w)?.checked_div(&t_before)?;
        composed = composed.geo(&cga::dilator(&rho_c));
        product = &product * &rho_c;
        channels.push((*name, rho_c));
    }
    let closed = cga::projective_ratio(&composed, &observed).is_some();
    let residual = rho.checked_div(&product)?;
    Some(PoleFlux {
        t0,
        t1,
        rho,
        channels,
        residual,
        closed,
    })
}
