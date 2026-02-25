//! Property list with inheritance support

use crate::properties::{PropertyId, PropertyValue};
use fop_types::{FopError, Result};
use std::cell::RefCell;
use std::collections::HashMap;

/// Maximum property ID (294 properties in XSL-FO 1.1 + extensions)
const MAX_PROPERTY_ID: usize = 295;

/// Property inheritance flags
/// Based on XSL-FO 1.1 spec: http://www.w3.org/TR/xsl11/#property-index
static INHERITABLE_PROPERTIES: [bool; MAX_PROPERTY_ID] = {
    let mut flags = [false; MAX_PROPERTY_ID];

    // Common inheritable properties
    flags[PropertyId::Color as usize] = true;
    flags[PropertyId::FontFamily as usize] = true;
    flags[PropertyId::FontSize as usize] = true;
    flags[PropertyId::FontStyle as usize] = true;
    flags[PropertyId::FontWeight as usize] = true;
    flags[PropertyId::LineHeight as usize] = true;
    flags[PropertyId::TextAlign as usize] = true;
    flags[PropertyId::TextIndent as usize] = true;
    flags[PropertyId::WhiteSpace as usize] = true;
    flags[PropertyId::Visibility as usize] = true;
    flags[PropertyId::LetterSpacing as usize] = true;
    flags[PropertyId::WordSpacing as usize] = true;
    flags[PropertyId::Direction as usize] = true;
    flags[PropertyId::WritingMode as usize] = true;

    // Add more inheritable properties as needed
    // (This is a simplified set; full spec has more)

    flags
};

/// A list of properties with inheritance support
///
/// Properties can be:
/// 1. Explicitly set on this element
/// 2. Inherited from parent (if inheritable)
/// 3. Use initial/default value
pub struct PropertyList<'a> {
    /// Explicitly set properties (sparse array, None = not set)
    explicit: Box<[Option<PropertyValue>]>,

    /// Parent property list for inheritance
    parent: Option<&'a PropertyList<'a>>,

    /// Cache for computed/inherited values to avoid repeated lookups
    computed_cache: RefCell<HashMap<PropertyId, PropertyValue>>,
}

impl<'a> PropertyList<'a> {
    /// Create a new empty property list
    pub fn new() -> Self {
        Self {
            explicit: vec![None; MAX_PROPERTY_ID].into_boxed_slice(),
            parent: None,
            computed_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Create a new property list with a parent for inheritance
    pub fn with_parent(parent: &'a PropertyList<'a>) -> Self {
        Self {
            explicit: vec![None; MAX_PROPERTY_ID].into_boxed_slice(),
            parent: Some(parent),
            computed_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Set an explicit property value
    pub fn set(&mut self, id: PropertyId, value: PropertyValue) {
        let index = id as usize;
        if index < MAX_PROPERTY_ID {
            self.explicit[index] = Some(value);
            // Invalidate cache for this property
            self.computed_cache.borrow_mut().remove(&id);
        }
    }

    /// Get a property value, considering inheritance
    ///
    /// Resolution order:
    /// 1. Explicit value on this element (if it's "inherit", walk parent chain)
    /// 2. Inherited value from parent (if property is inheritable)
    /// 3. Initial/default value
    pub fn get(&self, id: PropertyId) -> Result<PropertyValue> {
        let index = id as usize;
        if index >= MAX_PROPERTY_ID {
            return Err(FopError::UnknownProperty(format!("{:?}", id)));
        }

        // Check cache first
        if let Some(cached) = self.computed_cache.borrow().get(&id) {
            return Ok(cached.clone());
        }

        // Check explicit value
        if let Some(ref value) = self.explicit[index] {
            // If explicit value is "inherit", get from parent
            if value.is_inherit() {
                if let Some(parent) = self.parent {
                    // Recursively get from parent (handles nested inherits)
                    if let Ok(inherited) = parent.get(id) {
                        self.computed_cache
                            .borrow_mut()
                            .insert(id, inherited.clone());
                        return Ok(inherited);
                    }
                }
                // No parent - use initial value
                let initial = self.get_initial_value(id);
                self.computed_cache.borrow_mut().insert(id, initial.clone());
                return Ok(initial);
            }

            // Not inherit - cache and return
            self.computed_cache.borrow_mut().insert(id, value.clone());
            return Ok(value.clone());
        }

        // Check if inheritable and has parent
        if INHERITABLE_PROPERTIES[index] {
            if let Some(parent) = self.parent {
                if let Ok(inherited) = parent.get(id) {
                    // Cache inherited value
                    self.computed_cache
                        .borrow_mut()
                        .insert(id, inherited.clone());
                    return Ok(inherited);
                }
            }
        }

        // Return initial value
        let initial = self.get_initial_value(id);
        self.computed_cache.borrow_mut().insert(id, initial.clone());
        Ok(initial)
    }

    /// Get the initial/default value for a property
    fn get_initial_value(&self, id: PropertyId) -> PropertyValue {
        crate::properties::get_initial_value(id)
    }

    /// Check if a property is explicitly set
    pub fn is_explicit(&self, id: PropertyId) -> bool {
        let index = id as usize;
        index < MAX_PROPERTY_ID && self.explicit[index].is_some()
    }

    /// Check if a property is inheritable
    pub fn is_inheritable(id: PropertyId) -> bool {
        let index = id as usize;
        index < MAX_PROPERTY_ID && INHERITABLE_PROPERTIES[index]
    }

    /// Get the parent property list
    pub fn parent(&self) -> Option<&'a PropertyList<'a>> {
        self.parent
    }

    /// Validate all explicitly set properties
    ///
    /// This method checks all properties that are explicitly set on this element
    /// against their validation rules defined in the XSL-FO specification.
    pub fn validate(&self) -> Result<()> {
        use crate::properties::validation::PropertyValidator;

        let validator = PropertyValidator::new();

        // Iterate through all explicitly set properties
        for (index, opt_value) in self.explicit.iter().enumerate() {
            if let Some(value) = opt_value {
                // Convert index to PropertyId
                // Safety: we only store valid PropertyIds in the array
                let property_id: PropertyId = unsafe { std::mem::transmute(index as u16) };

                // Validate the property value
                validator.validate(property_id, value)?;
            }
        }

        Ok(())
    }
}

impl<'a> Default for PropertyList<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_types::{Color, Length};

    #[test]
    fn test_explicit_property() {
        let mut props = PropertyList::new();
        props.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(14.0)),
        );

        let value = props
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(value.as_length(), Some(Length::from_pt(14.0)));
    }

    #[test]
    fn test_property_inheritance() {
        // Parent with font-size set
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(16.0)),
        );

        // Child should inherit font-size
        let child = PropertyList::with_parent(&parent);
        let value = child
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(value.as_length(), Some(Length::from_pt(16.0)));
    }

    #[test]
    fn test_inheritance_override() {
        // Parent with font-size set
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(16.0)),
        );

        // Child overrides font-size
        let mut child = PropertyList::with_parent(&parent);
        child.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(14.0)),
        );

        let value = child
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(value.as_length(), Some(Length::from_pt(14.0)));
    }

    #[test]
    fn test_initial_value() {
        let props = PropertyList::new();

        // Should get initial value for font-size
        let value = props
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(value.as_length(), Some(Length::from_pt(12.0)));
    }

    #[test]
    fn test_non_inheritable_property() {
        // Parent with margin-top set (non-inheritable)
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::MarginTop,
            PropertyValue::Length(Length::from_pt(20.0)),
        );

        // Child should NOT inherit margin-top
        let child = PropertyList::with_parent(&parent);
        let value = child
            .get(PropertyId::MarginTop)
            .expect("test: should succeed");

        // Should be initial value (0pt), not inherited
        assert_eq!(value.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_color_inheritance() {
        let mut parent = PropertyList::new();
        parent.set(PropertyId::Color, PropertyValue::Color(Color::RED));

        let child = PropertyList::with_parent(&parent);
        let value = child.get(PropertyId::Color).expect("test: should succeed");
        assert_eq!(value.as_color(), Some(Color::RED));
    }

    #[test]
    fn test_inheritance_chain() {
        // Grandparent
        let mut grandparent = PropertyList::new();
        grandparent.set(
            PropertyId::FontFamily,
            PropertyValue::String(std::borrow::Cow::Borrowed("Arial")),
        );

        // Parent (doesn't set font-family)
        let parent = PropertyList::with_parent(&grandparent);

        // Child should inherit from grandparent
        let child = PropertyList::with_parent(&parent);
        let value = child
            .get(PropertyId::FontFamily)
            .expect("test: should succeed");
        assert_eq!(value.as_string(), Some("Arial"));
    }

    // ===== EDGE CASE TESTS FOR PROPERTY INHERITANCE =====

    #[test]
    fn test_explicit_inherit_keyword() {
        // Parent with margin (non-inheritable)
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::MarginTop,
            PropertyValue::Length(Length::from_pt(20.0)),
        );

        // Child explicitly requests to inherit margin-top (using Inherit variant)
        let mut child = PropertyList::with_parent(&parent);
        child.set(PropertyId::MarginTop, PropertyValue::Inherit);

        // Should inherit from parent even though margin is normally non-inheritable
        let value = child
            .get(PropertyId::MarginTop)
            .expect("test: should succeed");
        assert_eq!(value.as_length(), Some(Length::from_pt(20.0)));
    }

    #[test]
    fn test_inherit_with_no_parent() {
        // Child with no parent explicitly requests inheritance
        let mut child = PropertyList::new();
        child.set(PropertyId::FontSize, PropertyValue::Inherit);

        // Should fall back to initial value when no parent exists
        let value = child
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(value.as_length(), Some(Length::from_pt(12.0)));
    }

    #[test]
    fn test_percentage_inheritance() {
        use fop_types::Percentage;

        // Parent with percentage value
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Percentage(Percentage::new(150.0)),
        );

        // Child should inherit the percentage
        let child = PropertyList::with_parent(&parent);
        let value = child
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(value.as_percentage(), Some(Percentage::new(150.0)));
    }

    #[test]
    fn test_enum_value_inheritance() {
        // Parent with text-align (enum property)
        let mut parent = PropertyList::new();
        parent.set(PropertyId::TextAlign, PropertyValue::Enum(23)); // CENTER

        // Child should inherit enum value
        let child = PropertyList::with_parent(&parent);
        let value = child
            .get(PropertyId::TextAlign)
            .expect("test: should succeed");
        assert_eq!(value.as_enum(), Some(23));
    }

    #[test]
    fn test_list_value_inheritance() {
        // Parent with font-family list
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::FontFamily,
            PropertyValue::List(vec![
                PropertyValue::String(std::borrow::Cow::Borrowed("Arial")),
                PropertyValue::String(std::borrow::Cow::Borrowed("sans-serif")),
            ]),
        );

        // Child should inherit the entire list
        let child = PropertyList::with_parent(&parent);
        let value = child
            .get(PropertyId::FontFamily)
            .expect("test: should succeed");
        match value {
            PropertyValue::List(families) => {
                assert_eq!(families.len(), 2);
                assert_eq!(families[0].as_string(), Some("Arial"));
                assert_eq!(families[1].as_string(), Some("sans-serif"));
            }
            _ => panic!("Expected List value"),
        }
    }

    #[test]
    fn test_deep_override_in_chain() {
        // Grandparent sets font-size
        let mut grandparent = PropertyList::new();
        grandparent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(20.0)),
        );

        // Parent overrides to smaller size
        let mut parent = PropertyList::with_parent(&grandparent);
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(14.0)),
        );

        // Child inherits from parent (not grandparent)
        let child = PropertyList::with_parent(&parent);
        let value = child
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(value.as_length(), Some(Length::from_pt(14.0)));
    }

    #[test]
    fn test_multiple_properties_same_element() {
        // Parent sets multiple properties
        let mut parent = PropertyList::new();
        parent.set(PropertyId::Color, PropertyValue::Color(Color::RED));
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(16.0)),
        );
        parent.set(
            PropertyId::TextAlign,
            PropertyValue::Enum(23), // CENTER
        );

        // Child should inherit all inheritable properties
        let child = PropertyList::with_parent(&parent);

        assert_eq!(
            child
                .get(PropertyId::Color)
                .expect("test: should succeed")
                .as_color(),
            Some(Color::RED)
        );
        assert_eq!(
            child
                .get(PropertyId::FontSize)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::from_pt(16.0))
        );
        assert_eq!(
            child
                .get(PropertyId::TextAlign)
                .expect("test: should succeed")
                .as_enum(),
            Some(23)
        );
    }

    #[test]
    fn test_partial_override_multiple_properties() {
        // Parent sets three properties
        let mut parent = PropertyList::new();
        parent.set(PropertyId::Color, PropertyValue::Color(Color::RED));
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(16.0)),
        );
        parent.set(
            PropertyId::FontFamily,
            PropertyValue::String(std::borrow::Cow::Borrowed("Arial")),
        );

        // Child overrides only font-size
        let mut child = PropertyList::with_parent(&parent);
        child.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(12.0)),
        );

        // Color and font-family should be inherited
        assert_eq!(
            child
                .get(PropertyId::Color)
                .expect("test: should succeed")
                .as_color(),
            Some(Color::RED)
        );
        assert_eq!(
            child
                .get(PropertyId::FontFamily)
                .expect("test: should succeed")
                .as_string(),
            Some("Arial")
        );

        // Font-size should be overridden
        assert_eq!(
            child
                .get(PropertyId::FontSize)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::from_pt(12.0))
        );
    }

    #[test]
    fn test_border_properties_not_inherited() {
        // Parent sets border properties (all non-inheritable)
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::BorderTopWidth,
            PropertyValue::Length(Length::from_pt(2.0)),
        );
        parent.set(PropertyId::BorderTopStyle, PropertyValue::Enum(123)); // SOLID
        parent.set(
            PropertyId::BorderTopColor,
            PropertyValue::Color(Color::BLUE),
        );

        // Child should NOT inherit any border properties
        let child = PropertyList::with_parent(&parent);

        // Should all be initial values, not inherited
        let width = child
            .get(PropertyId::BorderTopWidth)
            .expect("test: should succeed");
        assert_eq!(width.as_length(), Some(Length::from_pt(1.0))); // medium = 1pt

        let style = child
            .get(PropertyId::BorderTopStyle)
            .expect("test: should succeed");
        assert_eq!(style.as_enum(), Some(86)); // NONE

        let color = child
            .get(PropertyId::BorderTopColor)
            .expect("test: should succeed");
        assert_eq!(color.as_color(), Some(Color::BLACK)); // initial
    }

    #[test]
    fn test_mixed_inheritable_and_non_inheritable() {
        // Parent sets both inheritable and non-inheritable properties
        let mut parent = PropertyList::new();
        parent.set(PropertyId::Color, PropertyValue::Color(Color::GREEN)); // inheritable
        parent.set(
            PropertyId::MarginTop,
            PropertyValue::Length(Length::from_pt(10.0)),
        ); // non-inheritable
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(14.0)),
        ); // inheritable
        parent.set(
            PropertyId::PaddingLeft,
            PropertyValue::Length(Length::from_pt(5.0)),
        ); // non-inheritable

        let child = PropertyList::with_parent(&parent);

        // Inheritable properties should be inherited
        assert_eq!(
            child
                .get(PropertyId::Color)
                .expect("test: should succeed")
                .as_color(),
            Some(Color::GREEN)
        );
        assert_eq!(
            child
                .get(PropertyId::FontSize)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::from_pt(14.0))
        );

        // Non-inheritable properties should use initial values
        assert_eq!(
            child
                .get(PropertyId::MarginTop)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::ZERO)
        );
        assert_eq!(
            child
                .get(PropertyId::PaddingLeft)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::ZERO)
        );
    }

    #[test]
    fn test_cache_efficiency() {
        // Parent with several properties
        let mut parent = PropertyList::new();
        parent.set(PropertyId::Color, PropertyValue::Color(Color::RED));
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(16.0)),
        );

        let child = PropertyList::with_parent(&parent);

        // First access should populate cache
        let color1 = child.get(PropertyId::Color).expect("test: should succeed");
        assert_eq!(color1.as_color(), Some(Color::RED));

        // Second access should use cache
        let color2 = child.get(PropertyId::Color).expect("test: should succeed");
        assert_eq!(color2.as_color(), Some(Color::RED));

        // Verify both accesses return the same value
        assert_eq!(color1, color2);
    }

    #[test]
    fn test_empty_parent_chain_uses_initial_values() {
        // Child with no parent and no explicit properties
        let child = PropertyList::new();

        // All properties should return initial values
        assert_eq!(
            child
                .get(PropertyId::FontSize)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::from_pt(12.0))
        );
        assert_eq!(
            child
                .get(PropertyId::Color)
                .expect("test: should succeed")
                .as_color(),
            Some(Color::BLACK)
        );
        assert_eq!(
            child
                .get(PropertyId::MarginTop)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::ZERO)
        );
    }

    #[test]
    fn test_four_level_inheritance_chain() {
        // Great-grandparent
        let mut great_grandparent = PropertyList::new();
        great_grandparent.set(PropertyId::Color, PropertyValue::Color(Color::BLUE));

        // Grandparent (doesn't set color)
        let grandparent = PropertyList::with_parent(&great_grandparent);

        // Parent (doesn't set color)
        let parent = PropertyList::with_parent(&grandparent);

        // Child should inherit from great-grandparent
        let child = PropertyList::with_parent(&parent);
        let value = child.get(PropertyId::Color).expect("test: should succeed");
        assert_eq!(value.as_color(), Some(Color::BLUE));
    }

    #[test]
    fn test_override_after_multiple_gets() {
        // Parent with font-size
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(16.0)),
        );

        // Child gets inherited value multiple times
        let mut child = PropertyList::with_parent(&parent);
        assert_eq!(
            child
                .get(PropertyId::FontSize)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::from_pt(16.0))
        );
        assert_eq!(
            child
                .get(PropertyId::FontSize)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::from_pt(16.0))
        );

        // Now child overrides the property
        child.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(14.0)),
        );

        // Should now return the overridden value
        assert_eq!(
            child
                .get(PropertyId::FontSize)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::from_pt(14.0))
        );
    }
}

// ===== ADDITIONAL TESTS =====
#[cfg(test)]
mod additional_tests {
    use super::*;
    use fop_types::{Color, Length};

    #[test]
    fn test_property_list_new_is_empty_of_explicit() {
        let list = PropertyList::new();
        // No explicit properties set
        assert!(!list.is_explicit(PropertyId::FontSize));
        assert!(!list.is_explicit(PropertyId::Color));
        assert!(!list.is_explicit(PropertyId::MarginTop));
    }

    #[test]
    fn test_is_explicit_after_set() {
        let mut list = PropertyList::new();
        list.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(12.0)),
        );
        assert!(list.is_explicit(PropertyId::FontSize));
        assert!(!list.is_explicit(PropertyId::Color));
    }

    #[test]
    fn test_is_inheritable_font_size() {
        assert!(PropertyList::is_inheritable(PropertyId::FontSize));
    }

    #[test]
    fn test_is_inheritable_color() {
        assert!(PropertyList::is_inheritable(PropertyId::Color));
    }

    #[test]
    fn test_is_inheritable_font_family() {
        assert!(PropertyList::is_inheritable(PropertyId::FontFamily));
    }

    #[test]
    fn test_is_inheritable_text_align() {
        assert!(PropertyList::is_inheritable(PropertyId::TextAlign));
    }

    #[test]
    fn test_is_not_inheritable_margin_top() {
        assert!(!PropertyList::is_inheritable(PropertyId::MarginTop));
    }

    #[test]
    fn test_is_not_inheritable_margin_bottom() {
        assert!(!PropertyList::is_inheritable(PropertyId::MarginBottom));
    }

    #[test]
    fn test_is_not_inheritable_border_top_style() {
        assert!(!PropertyList::is_inheritable(PropertyId::BorderTopStyle));
    }

    #[test]
    fn test_parent_method_returns_none_without_parent() {
        let list = PropertyList::new();
        assert!(list.parent().is_none());
    }

    #[test]
    fn test_parent_method_returns_some_with_parent() {
        let parent = PropertyList::new();
        let child = PropertyList::with_parent(&parent);
        assert!(child.parent().is_some());
    }

    #[test]
    fn test_default_is_same_as_new() {
        let list1 = PropertyList::new();
        let list2 = PropertyList::default();
        // Both should return the initial value for font-size
        assert_eq!(
            list1
                .get(PropertyId::FontSize)
                .expect("test: should succeed"),
            list2
                .get(PropertyId::FontSize)
                .expect("test: should succeed")
        );
    }

    #[test]
    fn test_set_overwrites_existing_value() {
        let mut list = PropertyList::new();
        list.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(10.0)),
        );
        list.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(20.0)),
        );
        let v = list
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(v.as_length(), Some(Length::from_pt(20.0)));
    }

    #[test]
    fn test_set_multiple_different_properties() {
        let mut list = PropertyList::new();
        list.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(14.0)),
        );
        list.set(PropertyId::Color, PropertyValue::Color(Color::RED));
        list.set(
            PropertyId::MarginTop,
            PropertyValue::Length(Length::from_pt(5.0)),
        );

        assert_eq!(
            list.get(PropertyId::FontSize)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::from_pt(14.0))
        );
        assert_eq!(
            list.get(PropertyId::Color)
                .expect("test: should succeed")
                .as_color(),
            Some(Color::RED)
        );
        assert_eq!(
            list.get(PropertyId::MarginTop)
                .expect("test: should succeed")
                .as_length(),
            Some(Length::from_pt(5.0))
        );
    }

    #[test]
    fn test_inheritance_chain_three_levels_color() {
        // Root has color
        let mut root = PropertyList::new();
        root.set(PropertyId::Color, PropertyValue::Color(Color::RED));

        // Middle level has no color set
        let middle = PropertyList::with_parent(&root);

        // Leaf should inherit from root through middle
        let leaf = PropertyList::with_parent(&middle);
        let v = leaf.get(PropertyId::Color).expect("test: should succeed");
        assert_eq!(v.as_color(), Some(Color::RED));
    }

    #[test]
    fn test_non_inheritable_not_propagated_two_levels() {
        // Root sets margin-top (non-inheritable)
        let mut root = PropertyList::new();
        root.set(
            PropertyId::MarginTop,
            PropertyValue::Length(Length::from_pt(30.0)),
        );

        // Middle inherits (but margin isn't inheritable, so shouldn't propagate)
        let middle = PropertyList::with_parent(&root);

        // Leaf should get initial value, not root's margin
        let leaf = PropertyList::with_parent(&middle);
        let v = leaf
            .get(PropertyId::MarginTop)
            .expect("test: should succeed");
        assert_eq!(v.as_length(), Some(Length::ZERO));
    }

    #[test]
    fn test_cache_is_populated_after_get() {
        let mut list = PropertyList::new();
        list.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(16.0)),
        );

        // Get twice - second should use cache
        let v1 = list
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        let v2 = list
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_initial_value_font_weight() {
        let list = PropertyList::new();
        // font-weight initial is "normal" (enum value)
        let v = list
            .get(PropertyId::FontWeight)
            .expect("test: should succeed");
        assert!(v.as_enum().is_some());
    }

    #[test]
    fn test_initial_value_font_style() {
        let list = PropertyList::new();
        // font-style initial is "normal"
        let v = list
            .get(PropertyId::FontStyle)
            .expect("test: should succeed");
        assert!(v.as_enum().is_some());
    }

    #[test]
    fn test_initial_value_overflow() {
        let list = PropertyList::new();
        let v = list
            .get(PropertyId::Overflow)
            .expect("test: should succeed");
        assert!(v.as_enum().is_some());
    }

    #[test]
    fn test_initial_value_white_space() {
        let list = PropertyList::new();
        let v = list
            .get(PropertyId::WhiteSpace)
            .expect("test: should succeed");
        assert!(v.as_enum().is_some());
    }

    #[test]
    fn test_validate_valid_properties() {
        let mut list = PropertyList::new();
        list.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(12.0)),
        );
        list.set(PropertyId::Color, PropertyValue::Color(Color::RED));
        assert!(
            list.validate().is_ok(),
            "Valid properties should validate OK"
        );
    }

    #[test]
    fn test_explicit_inherit_walks_to_grandparent() {
        // Grandparent has font-size
        let mut grandparent = PropertyList::new();
        grandparent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(24.0)),
        );

        // Parent does NOT have font-size - child will inherit from grandparent via parent
        let parent = PropertyList::with_parent(&grandparent);

        // Child requests explicit inherit - should walk up to grandparent
        let mut child = PropertyList::with_parent(&parent);
        child.set(PropertyId::FontSize, PropertyValue::Inherit);

        let v = child
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(v.as_length(), Some(Length::from_pt(24.0)));
    }

    #[test]
    fn test_with_parent_preserves_parent_explicit_values() {
        let mut parent = PropertyList::new();
        parent.set(
            PropertyId::FontFamily,
            PropertyValue::String(std::borrow::Cow::Borrowed("Helvetica")),
        );
        parent.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(10.0)),
        );

        let child = PropertyList::with_parent(&parent);

        // Child inherits both font-family (inheritable) and font-size (inheritable)
        let ff = child
            .get(PropertyId::FontFamily)
            .expect("test: should succeed");
        assert_eq!(ff.as_string(), Some("Helvetica"));

        let fs = child
            .get(PropertyId::FontSize)
            .expect("test: should succeed");
        assert_eq!(fs.as_length(), Some(Length::from_pt(10.0)));
    }
}
