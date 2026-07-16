//! Force-directed layout simulation for the 3D projector.
//!
//! Pure math: given a list of module IDs and a coupling matrix, produce a
//! deterministic 3D position per module. No I/O, no HTML, no dependence on
//! the projector or on serde. Kept separate so that the layout can be
//! tested and tuned independently of the rendering pipeline.

use std::collections::HashMap;

use super::model::ForceDirectedConfig;

/// Run force-directed layout simulation and return a position per module.
///
/// The initial layout places modules around a helical arrangement; each
/// iteration applies pairwise Coulomb repulsion plus spring attraction
/// along edges above `min_edge_strength`. Positions are recentered before
/// return so the scene sits near the origin.
pub fn run_force_simulation(
    modules: &[String],
    matrix: &[Vec<f64>],
    cfg: &ForceDirectedConfig,
) -> HashMap<String, [f64; 3]> {
    let n = modules.len();
    if n == 0 {
        return HashMap::new();
    }

    // Initialize positions in a circle (not a line!)
    let mut positions: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let angle = (i as f64 / n as f64) * 2.0 * std::f64::consts::PI;
            let radius = 5.0;
            [
                angle.cos() * radius,
                (i as f64 * 0.5) - (n as f64 * 0.25),
                angle.sin() * radius,
            ]
        })
        .collect();

    // Run force simulation
    for _iter in 0..cfg.iterations {
        let mut forces: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]; n];

        // Repulsion between all pairs
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = positions[j][0] - positions[i][0];
                let dy = positions[j][1] - positions[i][1];
                let dz = positions[j][2] - positions[i][2];
                let dist_sq = dx * dx + dy * dy + dz * dz + 0.01; // Avoid division by zero
                let dist = dist_sq.sqrt();

                // Repulsion force (Coulomb's law)
                let repulsion = cfg.repulsion / dist_sq;
                let fx = (dx / dist) * repulsion;
                let fy = (dy / dist) * repulsion;
                let fz = (dz / dist) * repulsion;

                forces[i][0] -= fx;
                forces[i][1] -= fy;
                forces[i][2] -= fz;
                forces[j][0] += fx;
                forces[j][1] += fy;
                forces[j][2] += fz;
            }
        }

        // Attraction along edges (coupling)
        for i in 0..n {
            for j in 0..n {
                if i != j && matrix[i][j] > cfg.min_edge_strength {
                    let coupling = matrix[i][j];
                    let dx = positions[j][0] - positions[i][0];
                    let dy = positions[j][1] - positions[i][1];
                    let dz = positions[j][2] - positions[i][2];
                    let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);

                    // Attraction force (spring)
                    let attraction = cfg.attraction * coupling * dist;
                    let fx = (dx / dist) * attraction;
                    let fy = (dy / dist) * attraction;
                    let fz = (dz / dist) * attraction;

                    forces[i][0] += fx;
                    forces[i][1] += fy;
                    forces[i][2] += fz;
                }
            }
        }

        // Apply forces with damping
        let damping = 0.85;
        let max_displacement = 0.5;
        for i in 0..n {
            for d in 0..3 {
                let displacement = (forces[i][d] * 0.01).clamp(-max_displacement, max_displacement);
                positions[i][d] += displacement * damping;
            }
        }
    }

    // Center the layout
    let mut center = [0.0, 0.0, 0.0];
    for pos in &positions {
        center[0] += pos[0];
        center[1] += pos[1];
        center[2] += pos[2];
    }
    for c in &mut center {
        *c /= n as f64;
    }
    for pos in &mut positions {
        pos[0] -= center[0];
        pos[1] -= center[1];
        pos[2] -= center[2];
    }

    // Build result map
    modules
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), positions[i]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_empty_map() {
        let cfg = ForceDirectedConfig::default();
        let out = run_force_simulation(&[], &[], &cfg);
        assert!(out.is_empty());
    }

    #[test]
    fn every_module_gets_a_position() {
        let cfg = ForceDirectedConfig {
            iterations: 5,
            ..Default::default()
        };
        let mods = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let matrix = vec![
            vec![0.0, 0.5, 0.0],
            vec![0.5, 0.0, 0.5],
            vec![0.0, 0.5, 0.0],
        ];
        let out = run_force_simulation(&mods, &matrix, &cfg);
        assert_eq!(out.len(), 3);
        for m in &mods {
            assert!(out.contains_key(m));
        }
    }
}
