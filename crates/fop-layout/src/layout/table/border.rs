//! Border conflict resolution for the collapsed border model

use super::types::{CollapsedBorder, TableLayout};

impl TableLayout {
    /// Resolve border conflicts for collapsed border model
    ///
    /// According to CSS2.1 Section 17.6.2, when two borders collapse:
    /// 1. Hidden takes precedence over all other styles
    /// 2. None has the lowest precedence
    /// 3. Wider borders take precedence over narrower ones
    /// 4. If widths are equal, style precedence is:
    ///    double > solid > dashed > dotted > ridge > outset > groove > inset
    /// 5. If width and style are equal, the border from the cell wins over
    ///    the one from the row, which wins over the table
    pub fn resolve_border_conflict(
        &self,
        border1: CollapsedBorder,
        border2: CollapsedBorder,
    ) -> CollapsedBorder {
        use crate::area::BorderStyle;

        // Rule 1: Hidden takes precedence
        if matches!(border1.style, BorderStyle::Hidden) {
            return border1;
        }
        if matches!(border2.style, BorderStyle::Hidden) {
            return border2;
        }

        // Rule 2: None has lowest precedence
        if matches!(border1.style, BorderStyle::None) && !matches!(border2.style, BorderStyle::None)
        {
            return border2;
        }
        if matches!(border2.style, BorderStyle::None) && !matches!(border1.style, BorderStyle::None)
        {
            return border1;
        }
        if matches!(border1.style, BorderStyle::None) && matches!(border2.style, BorderStyle::None)
        {
            return border1; // Both none, return either
        }

        // Rule 3: Wider border wins
        if border1.width > border2.width {
            return border1;
        }
        if border2.width > border1.width {
            return border2;
        }

        // Rule 4: If widths are equal, compare styles
        let style1_precedence = self.border_style_precedence(border1.style);
        let style2_precedence = self.border_style_precedence(border2.style);

        if style1_precedence > style2_precedence {
            return border1;
        }
        if style2_precedence > style1_precedence {
            return border2;
        }

        // Rule 5: If both width and style are equal, prefer border1
        // (assuming border1 comes from a more specific element)
        border1
    }

    /// Get border style precedence for conflict resolution
    ///
    /// Higher values have higher precedence.
    fn border_style_precedence(&self, style: crate::area::BorderStyle) -> u8 {
        use crate::area::BorderStyle;
        match style {
            BorderStyle::Hidden => 10, // Highest (but handled separately)
            BorderStyle::Double => 9,
            BorderStyle::Solid => 8,
            BorderStyle::Dashed => 7,
            BorderStyle::Dotted => 6,
            BorderStyle::Ridge => 5,
            BorderStyle::Outset => 4,
            BorderStyle::Groove => 3,
            BorderStyle::Inset => 2,
            BorderStyle::None => 0, // Lowest (but handled separately)
        }
    }
}
