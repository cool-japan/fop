//! Break and keep property extraction

use fop_core::{PropertyId, PropertyList};

use super::types::{BreakValue, Keep, KeepConstraint};

/// Extract keep constraints from properties
///
/// Extracts keep-together, keep-with-next, and keep-with-previous properties
/// from a PropertyList and returns a KeepConstraint struct.
///
/// # Examples
///
/// ```
/// use fop_core::{PropertyList, PropertyId, PropertyValue};
/// use fop_layout::layout::extract_keep_constraint;
/// use std::borrow::Cow;
///
/// let mut props = PropertyList::new();
/// props.set(PropertyId::KeepTogether, PropertyValue::String(Cow::Borrowed("always")));
///
/// let constraint = extract_keep_constraint(&props);
/// assert!(constraint.must_keep_together());
/// ```
pub fn extract_keep_constraint(properties: &PropertyList) -> KeepConstraint {
    let mut constraint = KeepConstraint::new();

    // Extract keep-together.within-page
    if let Ok(value) = properties.get(PropertyId::KeepTogether) {
        constraint.keep_together = Keep::from_property_value(&value);
    }

    // Extract keep-with-next.within-page
    if let Ok(value) = properties.get(PropertyId::KeepWithNext) {
        constraint.keep_with_next = Keep::from_property_value(&value);
    }

    // Extract keep-with-previous.within-page
    if let Ok(value) = properties.get(PropertyId::KeepWithPrevious) {
        constraint.keep_with_previous = Keep::from_property_value(&value);
    }

    constraint
}

/// Extract break-before property from properties
///
/// # Examples
///
/// ```
/// use fop_core::{PropertyList, PropertyId, PropertyValue};
/// use fop_layout::layout::extract_break_before;
/// use std::borrow::Cow;
///
/// let mut props = PropertyList::new();
/// props.set(PropertyId::BreakBefore, PropertyValue::String(Cow::Borrowed("page")));
///
/// let break_val = extract_break_before(&props);
/// assert!(break_val.forces_break());
/// ```
pub fn extract_break_before(properties: &PropertyList) -> BreakValue {
    properties
        .get(PropertyId::BreakBefore)
        .ok()
        .map(|v| BreakValue::from_property_value(&v))
        .unwrap_or(BreakValue::Auto)
}

/// Extract break-after property from properties
///
/// # Examples
///
/// ```
/// use fop_core::{PropertyList, PropertyId, PropertyValue};
/// use fop_layout::layout::extract_break_after;
/// use std::borrow::Cow;
///
/// let mut props = PropertyList::new();
/// props.set(PropertyId::BreakAfter, PropertyValue::String(Cow::Borrowed("page")));
///
/// let break_val = extract_break_after(&props);
/// assert!(break_val.forces_break());
/// ```
pub fn extract_break_after(properties: &PropertyList) -> BreakValue {
    properties
        .get(PropertyId::BreakAfter)
        .ok()
        .map(|v| BreakValue::from_property_value(&v))
        .unwrap_or(BreakValue::Auto)
}

/// Extract widows property from properties
///
/// Widows specifies the minimum number of lines that must be left at the
/// top of a page when a paragraph is broken across pages. The default is 2.
///
/// # Examples
///
/// ```
/// use fop_core::{PropertyList, PropertyId, PropertyValue};
/// use fop_layout::layout::extract_widows;
///
/// let mut props = PropertyList::new();
/// props.set(PropertyId::Widows, PropertyValue::Integer(3));
///
/// let widows = extract_widows(&props);
/// assert_eq!(widows, 3);
/// ```
pub fn extract_widows(properties: &PropertyList) -> i32 {
    properties
        .get(PropertyId::Widows)
        .ok()
        .and_then(|v| v.as_integer())
        .unwrap_or(2) // Default value per XSL-FO spec
}

/// Extract orphans property from properties
///
/// Orphans specifies the minimum number of lines that must be left at the
/// bottom of a page when a paragraph is broken across pages. The default is 2.
///
/// # Examples
///
/// ```
/// use fop_core::{PropertyList, PropertyId, PropertyValue};
/// use fop_layout::layout::extract_orphans;
///
/// let mut props = PropertyList::new();
/// props.set(PropertyId::Orphans, PropertyValue::Integer(3));
///
/// let orphans = extract_orphans(&props);
/// assert_eq!(orphans, 3);
/// ```
pub fn extract_orphans(properties: &PropertyList) -> i32 {
    properties
        .get(PropertyId::Orphans)
        .ok()
        .and_then(|v| v.as_integer())
        .unwrap_or(2) // Default value per XSL-FO spec
}
