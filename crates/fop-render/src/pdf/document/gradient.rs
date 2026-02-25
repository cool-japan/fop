//! PDF gradient shading object generation
//!
//! Functions to write gradient shading and interpolation function objects into a PDF byte stream.

use fop_types::{ColorStop, Gradient};

/// Write gradient shading objects to PDF
///
/// Generates two objects per gradient:
/// 1. Function object (Type 2 exponential interpolation function)
/// 2. Shading object (Type 2 for axial/linear, Type 3 for radial)
///
/// # Arguments
/// * `gradient` - The gradient definition
/// * `function_obj_id` - Object ID for the function
/// * `shading_obj_id` - Object ID for the shading dictionary
/// * `bytes` - Output buffer
/// * `xref_offsets` - Cross-reference table offsets
pub(super) fn write_gradient_objects(
    gradient: &Gradient,
    function_obj_id: usize,
    shading_obj_id: usize,
    bytes: &mut Vec<u8>,
    xref_offsets: &mut Vec<usize>,
) {
    match gradient {
        Gradient::Linear {
            start_point,
            end_point,
            color_stops,
        } => {
            // For linear gradients, we create a Type 2 shading (axial)
            // with a Type 2 function (exponential interpolation) for each color segment

            if color_stops.len() < 2 {
                return; // Need at least 2 stops
            }

            // Generate function object(s)
            // For simplicity, we'll use a stitching function if there are multiple segments
            write_gradient_function(function_obj_id, color_stops, bytes, xref_offsets);

            // Generate shading object (Type 2 - Axial)
            xref_offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n", shading_obj_id).as_bytes());
            bytes.extend_from_slice(b"<<\n");
            bytes.extend_from_slice(b"/ShadingType 2\n"); // Axial shading
            bytes.extend_from_slice(b"/ColorSpace /DeviceRGB\n");

            // Coords: [x0 y0 x1 y1] - start and end points in user space
            // Use normalized coordinates (0-100 range)
            bytes.extend_from_slice(
                format!(
                    "/Coords [{:.3} {:.3} {:.3} {:.3}]\n",
                    start_point.x.to_pt(),
                    start_point.y.to_pt(),
                    end_point.x.to_pt(),
                    end_point.y.to_pt()
                )
                .as_bytes(),
            );

            bytes.extend_from_slice(format!("/Function {} 0 R\n", function_obj_id).as_bytes());
            bytes.extend_from_slice(b"/Extend [true true]\n"); // Extend beyond gradient line
            bytes.extend_from_slice(b">>\n");
            bytes.extend_from_slice(b"endobj\n");
        }
        Gradient::Radial {
            center,
            radius,
            color_stops,
        } => {
            // For radial gradients, we create a Type 3 shading (radial)
            if color_stops.len() < 2 {
                return;
            }

            // Generate function object(s)
            write_gradient_function(function_obj_id, color_stops, bytes, xref_offsets);

            // Generate shading object (Type 3 - Radial)
            xref_offsets.push(bytes.len());
            bytes.extend_from_slice(format!("{} 0 obj\n", shading_obj_id).as_bytes());
            bytes.extend_from_slice(b"<<\n");
            bytes.extend_from_slice(b"/ShadingType 3\n"); // Radial shading
            bytes.extend_from_slice(b"/ColorSpace /DeviceRGB\n");

            // Coords: [x0 y0 r0 x1 y1 r1] - start circle and end circle
            // For a simple radial gradient, start radius is 0
            let center_x = center.x.to_pt();
            let center_y = center.y.to_pt();
            let end_radius = radius * 100.0; // Scale from normalized to user space

            bytes.extend_from_slice(
                format!(
                    "/Coords [{:.3} {:.3} 0 {:.3} {:.3} {:.3}]\n",
                    center_x, center_y, center_x, center_y, end_radius
                )
                .as_bytes(),
            );

            bytes.extend_from_slice(format!("/Function {} 0 R\n", function_obj_id).as_bytes());
            bytes.extend_from_slice(b"/Extend [true true]\n");
            bytes.extend_from_slice(b">>\n");
            bytes.extend_from_slice(b"endobj\n");
        }
    }
}

/// Write gradient interpolation function
///
/// Creates either a Type 2 (exponential) function for simple 2-color gradients,
/// or a Type 3 (stitching) function for multi-stop gradients.
fn write_gradient_function(
    function_obj_id: usize,
    color_stops: &[ColorStop],
    bytes: &mut Vec<u8>,
    xref_offsets: &mut Vec<usize>,
) {
    xref_offsets.push(bytes.len());
    bytes.extend_from_slice(format!("{} 0 obj\n", function_obj_id).as_bytes());
    bytes.extend_from_slice(b"<<\n");

    if color_stops.len() == 2 {
        // Simple Type 2 function (exponential interpolation)
        bytes.extend_from_slice(b"/FunctionType 2\n");
        bytes.extend_from_slice(b"/Domain [0 1]\n");
        bytes.extend_from_slice(b"/N 1\n"); // Linear interpolation

        let c0 = &color_stops[0].color;
        let c1 = &color_stops[1].color;

        bytes.extend_from_slice(
            format!(
                "/C0 [{:.3} {:.3} {:.3}]\n",
                c0.r_f32(),
                c0.g_f32(),
                c0.b_f32()
            )
            .as_bytes(),
        );
        bytes.extend_from_slice(
            format!(
                "/C1 [{:.3} {:.3} {:.3}]\n",
                c1.r_f32(),
                c1.g_f32(),
                c1.b_f32()
            )
            .as_bytes(),
        );
    } else {
        // Type 3 stitching function for multi-stop gradients
        bytes.extend_from_slice(b"/FunctionType 3\n");
        bytes.extend_from_slice(b"/Domain [0 1]\n");

        // Build bounds array (boundaries between segments)
        bytes.extend_from_slice(b"/Bounds [");
        for stop in color_stops.iter().skip(1).take(color_stops.len() - 2) {
            bytes.extend_from_slice(format!("{:.3} ", stop.offset).as_bytes());
        }
        bytes.extend_from_slice(b"]\n");

        // Build encode array (map domain to function domains)
        bytes.extend_from_slice(b"/Encode [");
        for _ in 0..color_stops.len() - 1 {
            bytes.extend_from_slice(b"0 1 ");
        }
        bytes.extend_from_slice(b"]\n");

        // Build functions array (one function per segment)
        bytes.extend_from_slice(b"/Functions [\n");
        for i in 0..color_stops.len() - 1 {
            let c0 = &color_stops[i].color;
            let c1 = &color_stops[i + 1].color;

            bytes.extend_from_slice(b"  <<\n");
            bytes.extend_from_slice(b"    /FunctionType 2\n");
            bytes.extend_from_slice(b"    /Domain [0 1]\n");
            bytes.extend_from_slice(b"    /N 1\n");
            bytes.extend_from_slice(
                format!(
                    "    /C0 [{:.3} {:.3} {:.3}]\n",
                    c0.r_f32(),
                    c0.g_f32(),
                    c0.b_f32()
                )
                .as_bytes(),
            );
            bytes.extend_from_slice(
                format!(
                    "    /C1 [{:.3} {:.3} {:.3}]\n",
                    c1.r_f32(),
                    c1.g_f32(),
                    c1.b_f32()
                )
                .as_bytes(),
            );
            bytes.extend_from_slice(b"  >>\n");
        }
        bytes.extend_from_slice(b"]\n");
    }

    bytes.extend_from_slice(b">>\n");
    bytes.extend_from_slice(b"endobj\n");
}
