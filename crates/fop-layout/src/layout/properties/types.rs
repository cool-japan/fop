//! Core property types: Keep, KeepConstraint, BreakValue, OverflowBehavior

use std::fmt;

/// Keep values as per XSL-FO specification
///
/// Controls page and column breaking behavior to prevent orphans and widows.
///
/// # Examples
///
/// ```
/// use fop_layout::layout::Keep;
///
/// let keep = Keep::Always;
/// assert!(keep.is_active());
/// assert_eq!(keep.strength(), i32::MAX);
///
/// let keep_auto = Keep::Auto;
/// assert!(!keep_auto.is_active());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Keep {
    /// auto - no keep constraint
    #[default]
    Auto,
    /// always - always keep together/with-next/with-previous
    Always,
    /// Integer strength value (higher = stronger constraint)
    Integer(i32),
}

impl Keep {
    /// Create a Keep value from a property value
    pub fn from_property_value(value: &fop_core::PropertyValue) -> Self {
        if value.is_auto() {
            Keep::Auto
        } else if let Some(s) = value.as_string() {
            if s == "always" {
                Keep::Always
            } else {
                Keep::Auto
            }
        } else if let Some(i) = value.as_integer() {
            Keep::Integer(i)
        } else {
            Keep::Auto
        }
    }

    /// Check if this is a keep constraint (not Auto)
    pub fn is_active(&self) -> bool {
        !matches!(self, Keep::Auto)
    }

    /// Get the strength of the keep constraint (higher = stronger)
    pub fn strength(&self) -> i32 {
        match self {
            Keep::Auto => 0,
            Keep::Always => i32::MAX,
            Keep::Integer(i) => *i,
        }
    }
}

impl fmt::Display for Keep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Keep::Auto => write!(f, "auto"),
            Keep::Always => write!(f, "always"),
            Keep::Integer(i) => write!(f, "{}", i),
        }
    }
}

/// Keep constraint for page breaking
///
/// Represents the XSL-FO keep properties that control page breaking behavior.
///
/// # Examples
///
/// ```
/// use fop_layout::layout::{Keep, KeepConstraint};
///
/// let mut constraint = KeepConstraint::new();
/// constraint.keep_together = Keep::Always;
/// assert!(constraint.must_keep_together());
///
/// constraint.keep_with_next = Keep::Always;
/// assert!(constraint.must_keep_with_next());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct KeepConstraint {
    /// keep-together.within-page
    pub keep_together: Keep,
    /// keep-with-next.within-page
    pub keep_with_next: Keep,
    /// keep-with-previous.within-page
    pub keep_with_previous: Keep,
}

impl KeepConstraint {
    /// Create a new empty keep constraint
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any keep constraint is active
    pub fn has_constraint(&self) -> bool {
        self.keep_together.is_active()
            || self.keep_with_next.is_active()
            || self.keep_with_previous.is_active()
    }

    /// Check if keep-together is active
    pub fn must_keep_together(&self) -> bool {
        self.keep_together.is_active()
    }

    /// Check if keep-with-next is active
    pub fn must_keep_with_next(&self) -> bool {
        self.keep_with_next.is_active()
    }

    /// Check if keep-with-previous is active
    pub fn must_keep_with_previous(&self) -> bool {
        self.keep_with_previous.is_active()
    }
}

/// Break-before and break-after property values
///
/// Controls forced page/column breaks before or after an element.
/// Based on XSL-FO 1.1 specification.
///
/// # Examples
///
/// ```
/// use fop_layout::layout::BreakValue;
///
/// let break_val = BreakValue::Page;
/// assert!(break_val.forces_break());
///
/// let auto = BreakValue::Auto;
/// assert!(!auto.forces_break());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreakValue {
    /// auto - no forced break
    #[default]
    Auto,
    /// always - force break (deprecated, use page)
    Always,
    /// page - force page break
    Page,
    /// column - force column break
    Column,
    /// even-page - force break to next even-numbered page
    EvenPage,
    /// odd-page - force break to next odd-numbered page
    OddPage,
}

impl BreakValue {
    /// Create a BreakValue from a property value
    pub fn from_property_value(value: &fop_core::PropertyValue) -> Self {
        if value.is_auto() {
            BreakValue::Auto
        } else if let Some(s) = value.as_string() {
            match s {
                "always" => BreakValue::Always,
                "page" => BreakValue::Page,
                "column" => BreakValue::Column,
                "even-page" => BreakValue::EvenPage,
                "odd-page" => BreakValue::OddPage,
                _ => BreakValue::Auto,
            }
        } else {
            BreakValue::Auto
        }
    }

    /// Check if this value forces a break
    pub fn forces_break(&self) -> bool {
        !matches!(self, BreakValue::Auto)
    }

    /// Check if this forces a page break (not column)
    pub fn forces_page_break(&self) -> bool {
        matches!(
            self,
            BreakValue::Always | BreakValue::Page | BreakValue::EvenPage | BreakValue::OddPage
        )
    }

    /// Check if this requires an even page
    pub fn requires_even_page(&self) -> bool {
        matches!(self, BreakValue::EvenPage)
    }

    /// Check if this requires an odd page
    pub fn requires_odd_page(&self) -> bool {
        matches!(self, BreakValue::OddPage)
    }
}

impl fmt::Display for BreakValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BreakValue::Auto => write!(f, "auto"),
            BreakValue::Always => write!(f, "always"),
            BreakValue::Page => write!(f, "page"),
            BreakValue::Column => write!(f, "column"),
            BreakValue::EvenPage => write!(f, "even-page"),
            BreakValue::OddPage => write!(f, "odd-page"),
        }
    }
}

/// Overflow behavior values
///
/// Controls how content that overflows a container should be handled.
/// Based on XSL-FO and CSS overflow property.
///
/// # Examples
///
/// ```
/// use fop_layout::layout::OverflowBehavior;
///
/// let overflow = OverflowBehavior::Hidden;
/// assert!(overflow.clips_content());
///
/// let visible = OverflowBehavior::Visible;
/// assert!(!visible.clips_content());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverflowBehavior {
    /// visible - content is not clipped and may overflow
    #[default]
    Visible,
    /// hidden - content is clipped to the container
    Hidden,
    /// scroll - content is clipped but scrollbars are shown (PDF renders as hidden)
    Scroll,
    /// auto - browser decides (PDF renders as visible)
    Auto,
}

impl OverflowBehavior {
    /// Create an OverflowBehavior from a property value
    pub fn from_property_value(value: &fop_core::PropertyValue) -> Self {
        if let Some(s) = value.as_string() {
            match s {
                "hidden" => OverflowBehavior::Hidden,
                "scroll" => OverflowBehavior::Scroll,
                "auto" => OverflowBehavior::Auto,
                "visible" => OverflowBehavior::Visible,
                _ => OverflowBehavior::Visible,
            }
        } else {
            OverflowBehavior::Visible
        }
    }

    /// Check if this overflow behavior requires clipping
    pub fn clips_content(&self) -> bool {
        matches!(self, OverflowBehavior::Hidden | OverflowBehavior::Scroll)
    }
}

impl fmt::Display for OverflowBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OverflowBehavior::Visible => write!(f, "visible"),
            OverflowBehavior::Hidden => write!(f, "hidden"),
            OverflowBehavior::Scroll => write!(f, "scroll"),
            OverflowBehavior::Auto => write!(f, "auto"),
        }
    }
}
