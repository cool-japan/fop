//! Property ID constants generated from Apache FOP Constants.java

/// Property identifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum PropertyId {
    /// absolute-position
    AbsolutePosition = 1,
    /// active-state
    ActiveState = 2,
    /// alignment-adjust
    AlignmentAdjust = 3,
    /// alignment-baseline
    AlignmentBaseline = 4,
    /// auto-restore
    AutoRestore = 5,
    /// azimuth
    Azimuth = 6,
    /// background
    Background = 7,
    /// background-attachment
    BackgroundAttachment = 8,
    /// background-color
    BackgroundColor = 9,
    /// background-image
    BackgroundImage = 10,
    /// background-position
    BackgroundPosition = 11,
    /// background-position-horizontal
    BackgroundPositionHorizontal = 12,
    /// background-position-vertical
    BackgroundPositionVertical = 13,
    /// background-repeat
    BackgroundRepeat = 14,
    /// baseline-shift
    BaselineShift = 15,
    /// blank-or-not-blank
    BlankOrNotBlank = 16,
    /// block-progression-dimension
    BlockProgressionDimension = 17,
    /// border
    Border = 18,
    /// border-after-color
    BorderAfterColor = 19,
    /// border-after-precedence
    BorderAfterPrecedence = 20,
    /// border-after-style
    BorderAfterStyle = 21,
    /// border-after-width
    BorderAfterWidth = 22,
    /// border-before-color
    BorderBeforeColor = 23,
    /// border-before-precedence
    BorderBeforePrecedence = 24,
    /// border-before-style
    BorderBeforeStyle = 25,
    /// border-before-width
    BorderBeforeWidth = 26,
    /// border-bottom
    BorderBottom = 27,
    /// border-bottom-color
    BorderBottomColor = 28,
    /// border-bottom-style
    BorderBottomStyle = 29,
    /// border-bottom-width
    BorderBottomWidth = 30,
    /// border-collapse
    BorderCollapse = 31,
    /// border-color
    BorderColor = 32,
    /// border-end-color
    BorderEndColor = 33,
    /// border-end-precedence
    BorderEndPrecedence = 34,
    /// border-end-style
    BorderEndStyle = 35,
    /// border-end-width
    BorderEndWidth = 36,
    /// border-left
    BorderLeft = 37,
    /// border-left-color
    BorderLeftColor = 38,
    /// border-left-style
    BorderLeftStyle = 39,
    /// border-left-width
    BorderLeftWidth = 40,
    /// border-right
    BorderRight = 41,
    /// border-right-color
    BorderRightColor = 42,
    /// border-right-style
    BorderRightStyle = 43,
    /// border-right-width
    BorderRightWidth = 44,
    /// border-separation
    BorderSeparation = 45,
    /// border-spacing
    BorderSpacing = 46,
    /// border-start-color
    BorderStartColor = 47,
    /// border-start-precedence
    BorderStartPrecedence = 48,
    /// border-start-style
    BorderStartStyle = 49,
    /// border-start-width
    BorderStartWidth = 50,
    /// border-style
    BorderStyle = 51,
    /// border-top
    BorderTop = 52,
    /// border-top-color
    BorderTopColor = 53,
    /// border-top-style
    BorderTopStyle = 54,
    /// border-top-width
    BorderTopWidth = 55,
    /// border-width
    BorderWidth = 56,
    /// bottom
    Bottom = 57,
    /// break-after
    BreakAfter = 58,
    /// break-before
    BreakBefore = 59,
    /// caption-side
    CaptionSide = 60,
    /// case-name
    CaseName = 61,
    /// case-title
    CaseTitle = 62,
    /// change-bar-class
    ChangeBarClass = 63,
    /// change-bar-color
    ChangeBarColor = 64,
    /// change-bar-offset
    ChangeBarOffset = 65,
    /// change-bar-placement
    ChangeBarPlacement = 66,
    /// change-bar-style
    ChangeBarStyle = 67,
    /// change-bar-width
    ChangeBarWidth = 68,
    /// character
    Character = 69,
    /// clear
    Clear = 70,
    /// clip
    Clip = 71,
    /// color
    Color = 72,
    /// color-profile-name
    ColorProfileName = 73,
    /// column-count
    ColumnCount = 74,
    /// column-gap
    ColumnGap = 75,
    /// column-number
    ColumnNumber = 76,
    /// column-width
    ColumnWidth = 77,
    /// content-height
    ContentHeight = 78,
    /// content-type
    ContentType = 79,
    /// content-width
    ContentWidth = 80,
    /// country
    Country = 81,
    /// cue
    Cue = 82,
    /// cue-after
    CueAfter = 83,
    /// cue-before
    CueBefore = 84,
    /// destination-placement-offset
    DestinationPlacementOffset = 85,
    /// direction
    Direction = 86,
    /// display-align
    DisplayAlign = 87,
    /// dominant-baseline
    DominantBaseline = 88,
    /// elevation
    Elevation = 89,
    /// empty-cells
    EmptyCells = 90,
    /// end-indent
    EndIndent = 91,
    /// ends-row
    EndsRow = 92,
    /// extent
    Extent = 93,
    /// external-destination
    ExternalDestination = 94,
    /// float
    Float = 95,
    /// flow-map-name
    FlowMapName = 96,
    /// flow-map-reference
    FlowMapReference = 97,
    /// flow-name
    FlowName = 98,
    /// flow-name-reference
    FlowNameReference = 99,
    /// font
    Font = 100,
    /// font-family
    FontFamily = 101,
    /// font-selection-strategy
    FontSelectionStrategy = 102,
    /// font-size
    FontSize = 103,
    /// font-size-adjust
    FontSizeAdjust = 104,
    /// font-stretch
    FontStretch = 105,
    /// font-style
    FontStyle = 106,
    /// font-variant
    FontVariant = 107,
    /// font-weight
    FontWeight = 108,
    /// force-page-count
    ForcePageCount = 109,
    /// format
    Format = 110,
    /// glyph-orientation-horizontal
    GlyphOrientationHorizontal = 111,
    /// glyph-orientation-vertical
    GlyphOrientationVertical = 112,
    /// grouping-separator
    GroupingSeparator = 113,
    /// grouping-size
    GroupingSize = 114,
    /// height
    Height = 115,
    /// hyphenate
    Hyphenate = 116,
    /// hyphenation-character
    HyphenationCharacter = 117,
    /// hyphenation-keep
    HyphenationKeep = 118,
    /// hyphenation-ladder-count
    HyphenationLadderCount = 119,
    /// hyphenation-push-character-count
    HyphenationPushCharacterCount = 120,
    /// hyphenation-remain-character-count
    HyphenationRemainCharacterCount = 121,
    /// id
    Id = 122,
    /// indicate-destination
    IndicateDestination = 123,
    /// index-class
    IndexClass = 124,
    /// index-key
    IndexKey = 125,
    /// initial-page-number
    InitialPageNumber = 126,
    /// inline-progression-dimension
    InlineProgressionDimension = 127,
    /// internal-destination
    InternalDestination = 128,
    /// intrinsic-scale-value
    IntrinsicScaleValue = 129,
    /// intrusion-displace
    IntrusionDisplace = 130,
    /// keep-together
    KeepTogether = 131,
    /// keep-with-next
    KeepWithNext = 132,
    /// keep-with-previous
    KeepWithPrevious = 133,
    /// language
    Language = 134,
    /// last-line-end-indent
    LastLineEndIndent = 135,
    /// leader-alignment
    LeaderAlignment = 136,
    /// leader-length
    LeaderLength = 137,
    /// leader-pattern
    LeaderPattern = 138,
    /// leader-pattern-width
    LeaderPatternWidth = 139,
    /// left
    Left = 140,
    /// letter-spacing
    LetterSpacing = 141,
    /// letter-value
    LetterValue = 142,
    /// linefeed-treatment
    LinefeedTreatment = 143,
    /// line-height
    LineHeight = 144,
    /// line-height-shift-adjustment
    LineHeightShiftAdjustment = 145,
    /// line-stacking-strategy
    LineStackingStrategy = 146,
    /// margin
    Margin = 147,
    /// margin-bottom
    MarginBottom = 148,
    /// margin-left
    MarginLeft = 149,
    /// margin-right
    MarginRight = 150,
    /// margin-top
    MarginTop = 151,
    /// marker-class-name
    MarkerClassName = 152,
    /// master-name
    MasterName = 153,
    /// master-reference
    MasterReference = 154,
    /// max-height
    MaxHeight = 155,
    /// maximum-repeats
    MaximumRepeats = 156,
    /// max-width
    MaxWidth = 157,
    /// merge-pages-across-index-key-references
    MergePagesAcrossIndexKeyReferences = 158,
    /// merge-ranges-across-index-key-references
    MergeRangesAcrossIndexKeyReferences = 159,
    /// merge-sequential-page-numbers
    MergeSequentialPageNumbers = 160,
    /// media-usage
    MediaUsage = 161,
    /// min-height
    MinHeight = 162,
    /// min-width
    MinWidth = 163,
    /// number-columns-repeated
    NumberColumnsRepeated = 164,
    /// number-columns-spanned
    NumberColumnsSpanned = 165,
    /// number-rows-spanned
    NumberRowsSpanned = 166,
    /// odd-or-even
    OddOrEven = 167,
    /// orphans
    Orphans = 168,
    /// overflow
    Overflow = 169,
    /// padding
    Padding = 170,
    /// padding-after
    PaddingAfter = 171,
    /// padding-before
    PaddingBefore = 172,
    /// padding-bottom
    PaddingBottom = 173,
    /// padding-end
    PaddingEnd = 174,
    /// padding-left
    PaddingLeft = 175,
    /// padding-right
    PaddingRight = 176,
    /// padding-start
    PaddingStart = 177,
    /// padding-top
    PaddingTop = 178,
    /// page-break-after
    PageBreakAfter = 179,
    /// page-break-before
    PageBreakBefore = 180,
    /// page-break-inside
    PageBreakInside = 181,
    /// page-citation-strategy
    PageCitationStrategy = 182,
    /// page-height
    PageHeight = 183,
    /// page-number-treatment
    PageNumberTreatment = 184,
    /// page-position
    PagePosition = 185,
    /// page-width
    PageWidth = 186,
    /// pause
    Pause = 187,
    /// pause-after
    PauseAfter = 188,
    /// pause-before
    PauseBefore = 189,
    /// pitch
    Pitch = 190,
    /// pitch-range
    PitchRange = 191,
    /// play-during
    PlayDuring = 192,
    /// position
    Position = 193,
    /// precedence
    Precedence = 194,
    /// provisional-distance-between-starts
    ProvisionalDistanceBetweenStarts = 195,
    /// provisional-label-separation
    ProvisionalLabelSeparation = 196,
    /// reference-orientation
    ReferenceOrientation = 197,
    /// ref-id
    RefId = 198,
    /// region-name
    RegionName = 199,
    /// region-name-reference
    RegionNameReference = 200,
    /// ref-index-key
    RefIndexKey = 201,
    /// relative-align
    RelativeAlign = 202,
    /// relative-position
    RelativePosition = 203,
    /// rendering-intent
    RenderingIntent = 204,
    /// retrieve-boundary
    RetrieveBoundary = 205,
    /// retrieve-boundary-within-table
    RetrieveBoundaryWithinTable = 206,
    /// retrieve-class-name
    RetrieveClassName = 207,
    /// retrieve-position
    RetrievePosition = 208,
    /// retrieve-position-within-table
    RetrievePositionWithinTable = 209,
    /// richness
    Richness = 210,
    /// right
    Right = 211,
    /// role
    Role = 212,
    /// rule-style
    RuleStyle = 213,
    /// rule-thickness
    RuleThickness = 214,
    /// scaling
    Scaling = 215,
    /// scaling-method
    ScalingMethod = 216,
    /// score-spaces
    ScoreSpaces = 217,
    /// script
    Script = 218,
    /// show-destination
    ShowDestination = 219,
    /// size
    Size = 220,
    /// source-document
    SourceDocument = 221,
    /// space-after
    SpaceAfter = 222,
    /// space-before
    SpaceBefore = 223,
    /// space-end
    SpaceEnd = 224,
    /// space-start
    SpaceStart = 225,
    /// span
    Span = 226,
    /// speak
    Speak = 227,
    /// speak-header
    SpeakHeader = 228,
    /// speak-numeral
    SpeakNumeral = 229,
    /// speak-punctuation
    SpeakPunctuation = 230,
    /// speech-rate
    SpeechRate = 231,
    /// src
    Src = 232,
    /// start-indent
    StartIndent = 233,
    /// starting-state
    StartingState = 234,
    /// starts-row
    StartsRow = 235,
    /// stress
    Stress = 236,
    /// suppress-at-line-break
    SuppressAtLineBreak = 237,
    /// switch-to
    SwitchTo = 238,
    /// table-layout
    TableLayout = 239,
    /// table-omit-footer-at-break
    TableOmitFooterAtBreak = 240,
    /// table-omit-header-at-break
    TableOmitHeaderAtBreak = 241,
    /// target-presentation-context
    TargetPresentationContext = 242,
    /// target-processing-context
    TargetProcessingContext = 243,
    /// target-stylesheet
    TargetStylesheet = 244,
    /// text-align
    TextAlign = 245,
    /// text-align-last
    TextAlignLast = 246,
    /// text-altitude
    TextAltitude = 247,
    /// text-decoration
    TextDecoration = 248,
    /// text-depth
    TextDepth = 249,
    /// text-indent
    TextIndent = 250,
    /// text-shadow
    TextShadow = 251,
    /// text-transform
    TextTransform = 252,
    /// top
    Top = 253,
    /// treat-as-word-space
    TreatAsWordSpace = 254,
    /// unicode-bidi
    UnicodeBidi = 255,
    /// vertical-align
    VerticalAlign = 256,
    /// visibility
    Visibility = 257,
    /// voice-family
    VoiceFamily = 258,
    /// volume
    Volume = 259,
    /// white-space
    WhiteSpace = 260,
    /// white-space-collapse
    WhiteSpaceCollapse = 261,
    /// white-space-treatment
    WhiteSpaceTreatment = 262,
    /// widows
    Widows = 263,
    /// width
    Width = 264,
    /// word-spacing
    WordSpacing = 265,
    /// wrap-option
    WrapOption = 266,
    /// writing-mode
    WritingMode = 267,
    /// xml-lang
    XmlLang = 268,
    /// z-index
    ZIndex = 269,
    /// x-widow-content-limit
    XWidowContentLimit = 270,
    /// x-orphan-content-limit
    XOrphanContentLimit = 271,
    /// x-disable-column-balancing
    XDisableColumnBalancing = 272,
    /// x-alt-text
    XAltText = 273,
    /// x-xml-base
    XXmlBase = 274,
    /// x-border-before-radius-start
    XBorderBeforeRadiusStart = 275,
    /// x-border-before-radius-end
    XBorderBeforeRadiusEnd = 276,
    /// x-border-after-radius-start
    XBorderAfterRadiusStart = 277,
    /// x-border-after-radius-end
    XBorderAfterRadiusEnd = 278,
    /// x-border-start-radius-before
    XBorderStartRadiusBefore = 279,
    /// x-border-start-radius-after
    XBorderStartRadiusAfter = 280,
    /// x-border-end-radius-before
    XBorderEndRadiusBefore = 281,
    /// x-border-end-radius-after
    XBorderEndRadiusAfter = 282,
    /// x-border-radius
    XBorderRadius = 283,
    /// x-border-before-start-radius
    XBorderBeforeStartRadius = 284,
    /// x-border-before-end-radius
    XBorderBeforeEndRadius = 285,
    /// x-border-after-start-radius
    XBorderAfterStartRadius = 286,
    /// x-border-after-end-radius
    XBorderAfterEndRadius = 287,
    /// x-number-conversion-features
    XNumberConversionFeatures = 288,
    /// x-header-column
    XHeaderColumn = 289,
    /// x-layer
    XLayer = 290,
    /// x-auto-toggle
    XAutoToggle = 291,
    /// x-background-image-width
    XBackgroundImageWidth = 292,
    /// x-background-image-height
    XBackgroundImageHeight = 293,
    /// x-abbreviation
    XAbbreviation = 294,
    /// opacity
    Opacity = 295,
}

impl PropertyId {
    /// Get the property name as a string
    pub fn name(self) -> &'static str {
        match self {
            PropertyId::AbsolutePosition => "absolute-position",
            PropertyId::ActiveState => "active-state",
            PropertyId::AlignmentAdjust => "alignment-adjust",
            PropertyId::AlignmentBaseline => "alignment-baseline",
            PropertyId::AutoRestore => "auto-restore",
            PropertyId::Azimuth => "azimuth",
            PropertyId::Background => "background",
            PropertyId::BackgroundAttachment => "background-attachment",
            PropertyId::BackgroundColor => "background-color",
            PropertyId::BackgroundImage => "background-image",
            PropertyId::BackgroundPosition => "background-position",
            PropertyId::BackgroundPositionHorizontal => "background-position-horizontal",
            PropertyId::BackgroundPositionVertical => "background-position-vertical",
            PropertyId::BackgroundRepeat => "background-repeat",
            PropertyId::BaselineShift => "baseline-shift",
            PropertyId::BlankOrNotBlank => "blank-or-not-blank",
            PropertyId::BlockProgressionDimension => "block-progression-dimension",
            PropertyId::Border => "border",
            PropertyId::BorderAfterColor => "border-after-color",
            PropertyId::BorderAfterPrecedence => "border-after-precedence",
            PropertyId::BorderAfterStyle => "border-after-style",
            PropertyId::BorderAfterWidth => "border-after-width",
            PropertyId::BorderBeforeColor => "border-before-color",
            PropertyId::BorderBeforePrecedence => "border-before-precedence",
            PropertyId::BorderBeforeStyle => "border-before-style",
            PropertyId::BorderBeforeWidth => "border-before-width",
            PropertyId::BorderBottom => "border-bottom",
            PropertyId::BorderBottomColor => "border-bottom-color",
            PropertyId::BorderBottomStyle => "border-bottom-style",
            PropertyId::BorderBottomWidth => "border-bottom-width",
            PropertyId::BorderCollapse => "border-collapse",
            PropertyId::BorderColor => "border-color",
            PropertyId::BorderEndColor => "border-end-color",
            PropertyId::BorderEndPrecedence => "border-end-precedence",
            PropertyId::BorderEndStyle => "border-end-style",
            PropertyId::BorderEndWidth => "border-end-width",
            PropertyId::BorderLeft => "border-left",
            PropertyId::BorderLeftColor => "border-left-color",
            PropertyId::BorderLeftStyle => "border-left-style",
            PropertyId::BorderLeftWidth => "border-left-width",
            PropertyId::BorderRight => "border-right",
            PropertyId::BorderRightColor => "border-right-color",
            PropertyId::BorderRightStyle => "border-right-style",
            PropertyId::BorderRightWidth => "border-right-width",
            PropertyId::BorderSeparation => "border-separation",
            PropertyId::BorderSpacing => "border-spacing",
            PropertyId::BorderStartColor => "border-start-color",
            PropertyId::BorderStartPrecedence => "border-start-precedence",
            PropertyId::BorderStartStyle => "border-start-style",
            PropertyId::BorderStartWidth => "border-start-width",
            PropertyId::BorderStyle => "border-style",
            PropertyId::BorderTop => "border-top",
            PropertyId::BorderTopColor => "border-top-color",
            PropertyId::BorderTopStyle => "border-top-style",
            PropertyId::BorderTopWidth => "border-top-width",
            PropertyId::BorderWidth => "border-width",
            PropertyId::Bottom => "bottom",
            PropertyId::BreakAfter => "break-after",
            PropertyId::BreakBefore => "break-before",
            PropertyId::CaptionSide => "caption-side",
            PropertyId::CaseName => "case-name",
            PropertyId::CaseTitle => "case-title",
            PropertyId::ChangeBarClass => "change-bar-class",
            PropertyId::ChangeBarColor => "change-bar-color",
            PropertyId::ChangeBarOffset => "change-bar-offset",
            PropertyId::ChangeBarPlacement => "change-bar-placement",
            PropertyId::ChangeBarStyle => "change-bar-style",
            PropertyId::ChangeBarWidth => "change-bar-width",
            PropertyId::Character => "character",
            PropertyId::Clear => "clear",
            PropertyId::Clip => "clip",
            PropertyId::Color => "color",
            PropertyId::ColorProfileName => "color-profile-name",
            PropertyId::ColumnCount => "column-count",
            PropertyId::ColumnGap => "column-gap",
            PropertyId::ColumnNumber => "column-number",
            PropertyId::ColumnWidth => "column-width",
            PropertyId::ContentHeight => "content-height",
            PropertyId::ContentType => "content-type",
            PropertyId::ContentWidth => "content-width",
            PropertyId::Country => "country",
            PropertyId::Cue => "cue",
            PropertyId::CueAfter => "cue-after",
            PropertyId::CueBefore => "cue-before",
            PropertyId::DestinationPlacementOffset => "destination-placement-offset",
            PropertyId::Direction => "direction",
            PropertyId::DisplayAlign => "display-align",
            PropertyId::DominantBaseline => "dominant-baseline",
            PropertyId::Elevation => "elevation",
            PropertyId::EmptyCells => "empty-cells",
            PropertyId::EndIndent => "end-indent",
            PropertyId::EndsRow => "ends-row",
            PropertyId::Extent => "extent",
            PropertyId::ExternalDestination => "external-destination",
            PropertyId::Float => "float",
            PropertyId::FlowMapName => "flow-map-name",
            PropertyId::FlowMapReference => "flow-map-reference",
            PropertyId::FlowName => "flow-name",
            PropertyId::FlowNameReference => "flow-name-reference",
            PropertyId::Font => "font",
            PropertyId::FontFamily => "font-family",
            PropertyId::FontSelectionStrategy => "font-selection-strategy",
            PropertyId::FontSize => "font-size",
            PropertyId::FontSizeAdjust => "font-size-adjust",
            PropertyId::FontStretch => "font-stretch",
            PropertyId::FontStyle => "font-style",
            PropertyId::FontVariant => "font-variant",
            PropertyId::FontWeight => "font-weight",
            PropertyId::ForcePageCount => "force-page-count",
            PropertyId::Format => "format",
            PropertyId::GlyphOrientationHorizontal => "glyph-orientation-horizontal",
            PropertyId::GlyphOrientationVertical => "glyph-orientation-vertical",
            PropertyId::GroupingSeparator => "grouping-separator",
            PropertyId::GroupingSize => "grouping-size",
            PropertyId::Height => "height",
            PropertyId::Hyphenate => "hyphenate",
            PropertyId::HyphenationCharacter => "hyphenation-character",
            PropertyId::HyphenationKeep => "hyphenation-keep",
            PropertyId::HyphenationLadderCount => "hyphenation-ladder-count",
            PropertyId::HyphenationPushCharacterCount => "hyphenation-push-character-count",
            PropertyId::HyphenationRemainCharacterCount => "hyphenation-remain-character-count",
            PropertyId::Id => "id",
            PropertyId::IndicateDestination => "indicate-destination",
            PropertyId::IndexClass => "index-class",
            PropertyId::IndexKey => "index-key",
            PropertyId::InitialPageNumber => "initial-page-number",
            PropertyId::InlineProgressionDimension => "inline-progression-dimension",
            PropertyId::InternalDestination => "internal-destination",
            PropertyId::IntrinsicScaleValue => "intrinsic-scale-value",
            PropertyId::IntrusionDisplace => "intrusion-displace",
            PropertyId::KeepTogether => "keep-together",
            PropertyId::KeepWithNext => "keep-with-next",
            PropertyId::KeepWithPrevious => "keep-with-previous",
            PropertyId::Language => "language",
            PropertyId::LastLineEndIndent => "last-line-end-indent",
            PropertyId::LeaderAlignment => "leader-alignment",
            PropertyId::LeaderLength => "leader-length",
            PropertyId::LeaderPattern => "leader-pattern",
            PropertyId::LeaderPatternWidth => "leader-pattern-width",
            PropertyId::Left => "left",
            PropertyId::LetterSpacing => "letter-spacing",
            PropertyId::LetterValue => "letter-value",
            PropertyId::LinefeedTreatment => "linefeed-treatment",
            PropertyId::LineHeight => "line-height",
            PropertyId::LineHeightShiftAdjustment => "line-height-shift-adjustment",
            PropertyId::LineStackingStrategy => "line-stacking-strategy",
            PropertyId::Margin => "margin",
            PropertyId::MarginBottom => "margin-bottom",
            PropertyId::MarginLeft => "margin-left",
            PropertyId::MarginRight => "margin-right",
            PropertyId::MarginTop => "margin-top",
            PropertyId::MarkerClassName => "marker-class-name",
            PropertyId::MasterName => "master-name",
            PropertyId::MasterReference => "master-reference",
            PropertyId::MaxHeight => "max-height",
            PropertyId::MaximumRepeats => "maximum-repeats",
            PropertyId::MaxWidth => "max-width",
            PropertyId::MergePagesAcrossIndexKeyReferences => {
                "merge-pages-across-index-key-references"
            }
            PropertyId::MergeRangesAcrossIndexKeyReferences => {
                "merge-ranges-across-index-key-references"
            }
            PropertyId::MergeSequentialPageNumbers => "merge-sequential-page-numbers",
            PropertyId::MediaUsage => "media-usage",
            PropertyId::MinHeight => "min-height",
            PropertyId::MinWidth => "min-width",
            PropertyId::NumberColumnsRepeated => "number-columns-repeated",
            PropertyId::NumberColumnsSpanned => "number-columns-spanned",
            PropertyId::NumberRowsSpanned => "number-rows-spanned",
            PropertyId::OddOrEven => "odd-or-even",
            PropertyId::Orphans => "orphans",
            PropertyId::Overflow => "overflow",
            PropertyId::Padding => "padding",
            PropertyId::PaddingAfter => "padding-after",
            PropertyId::PaddingBefore => "padding-before",
            PropertyId::PaddingBottom => "padding-bottom",
            PropertyId::PaddingEnd => "padding-end",
            PropertyId::PaddingLeft => "padding-left",
            PropertyId::PaddingRight => "padding-right",
            PropertyId::PaddingStart => "padding-start",
            PropertyId::PaddingTop => "padding-top",
            PropertyId::PageBreakAfter => "page-break-after",
            PropertyId::PageBreakBefore => "page-break-before",
            PropertyId::PageBreakInside => "page-break-inside",
            PropertyId::PageCitationStrategy => "page-citation-strategy",
            PropertyId::PageHeight => "page-height",
            PropertyId::PageNumberTreatment => "page-number-treatment",
            PropertyId::PagePosition => "page-position",
            PropertyId::PageWidth => "page-width",
            PropertyId::Pause => "pause",
            PropertyId::PauseAfter => "pause-after",
            PropertyId::PauseBefore => "pause-before",
            PropertyId::Pitch => "pitch",
            PropertyId::PitchRange => "pitch-range",
            PropertyId::PlayDuring => "play-during",
            PropertyId::Position => "position",
            PropertyId::Precedence => "precedence",
            PropertyId::ProvisionalDistanceBetweenStarts => "provisional-distance-between-starts",
            PropertyId::ProvisionalLabelSeparation => "provisional-label-separation",
            PropertyId::ReferenceOrientation => "reference-orientation",
            PropertyId::RefId => "ref-id",
            PropertyId::RegionName => "region-name",
            PropertyId::RegionNameReference => "region-name-reference",
            PropertyId::RefIndexKey => "ref-index-key",
            PropertyId::RelativeAlign => "relative-align",
            PropertyId::RelativePosition => "relative-position",
            PropertyId::RenderingIntent => "rendering-intent",
            PropertyId::RetrieveBoundary => "retrieve-boundary",
            PropertyId::RetrieveBoundaryWithinTable => "retrieve-boundary-within-table",
            PropertyId::RetrieveClassName => "retrieve-class-name",
            PropertyId::RetrievePosition => "retrieve-position",
            PropertyId::RetrievePositionWithinTable => "retrieve-position-within-table",
            PropertyId::Richness => "richness",
            PropertyId::Right => "right",
            PropertyId::Role => "role",
            PropertyId::RuleStyle => "rule-style",
            PropertyId::RuleThickness => "rule-thickness",
            PropertyId::Scaling => "scaling",
            PropertyId::ScalingMethod => "scaling-method",
            PropertyId::ScoreSpaces => "score-spaces",
            PropertyId::Script => "script",
            PropertyId::ShowDestination => "show-destination",
            PropertyId::Size => "size",
            PropertyId::SourceDocument => "source-document",
            PropertyId::SpaceAfter => "space-after",
            PropertyId::SpaceBefore => "space-before",
            PropertyId::SpaceEnd => "space-end",
            PropertyId::SpaceStart => "space-start",
            PropertyId::Span => "span",
            PropertyId::Speak => "speak",
            PropertyId::SpeakHeader => "speak-header",
            PropertyId::SpeakNumeral => "speak-numeral",
            PropertyId::SpeakPunctuation => "speak-punctuation",
            PropertyId::SpeechRate => "speech-rate",
            PropertyId::Src => "src",
            PropertyId::StartIndent => "start-indent",
            PropertyId::StartingState => "starting-state",
            PropertyId::StartsRow => "starts-row",
            PropertyId::Stress => "stress",
            PropertyId::SuppressAtLineBreak => "suppress-at-line-break",
            PropertyId::SwitchTo => "switch-to",
            PropertyId::TableLayout => "table-layout",
            PropertyId::TableOmitFooterAtBreak => "table-omit-footer-at-break",
            PropertyId::TableOmitHeaderAtBreak => "table-omit-header-at-break",
            PropertyId::TargetPresentationContext => "target-presentation-context",
            PropertyId::TargetProcessingContext => "target-processing-context",
            PropertyId::TargetStylesheet => "target-stylesheet",
            PropertyId::TextAlign => "text-align",
            PropertyId::TextAlignLast => "text-align-last",
            PropertyId::TextAltitude => "text-altitude",
            PropertyId::TextDecoration => "text-decoration",
            PropertyId::TextDepth => "text-depth",
            PropertyId::TextIndent => "text-indent",
            PropertyId::TextShadow => "text-shadow",
            PropertyId::TextTransform => "text-transform",
            PropertyId::Top => "top",
            PropertyId::TreatAsWordSpace => "treat-as-word-space",
            PropertyId::UnicodeBidi => "unicode-bidi",
            PropertyId::VerticalAlign => "vertical-align",
            PropertyId::Visibility => "visibility",
            PropertyId::VoiceFamily => "voice-family",
            PropertyId::Volume => "volume",
            PropertyId::WhiteSpace => "white-space",
            PropertyId::WhiteSpaceCollapse => "white-space-collapse",
            PropertyId::WhiteSpaceTreatment => "white-space-treatment",
            PropertyId::Widows => "widows",
            PropertyId::Width => "width",
            PropertyId::WordSpacing => "word-spacing",
            PropertyId::WrapOption => "wrap-option",
            PropertyId::WritingMode => "writing-mode",
            PropertyId::XmlLang => "xml-lang",
            PropertyId::ZIndex => "z-index",
            PropertyId::XWidowContentLimit => "x-widow-content-limit",
            PropertyId::XOrphanContentLimit => "x-orphan-content-limit",
            PropertyId::XDisableColumnBalancing => "x-disable-column-balancing",
            PropertyId::XAltText => "x-alt-text",
            PropertyId::XXmlBase => "x-xml-base",
            PropertyId::XBorderBeforeRadiusStart => "x-border-before-radius-start",
            PropertyId::XBorderBeforeRadiusEnd => "x-border-before-radius-end",
            PropertyId::XBorderAfterRadiusStart => "x-border-after-radius-start",
            PropertyId::XBorderAfterRadiusEnd => "x-border-after-radius-end",
            PropertyId::XBorderStartRadiusBefore => "x-border-start-radius-before",
            PropertyId::XBorderStartRadiusAfter => "x-border-start-radius-after",
            PropertyId::XBorderEndRadiusBefore => "x-border-end-radius-before",
            PropertyId::XBorderEndRadiusAfter => "x-border-end-radius-after",
            PropertyId::XBorderRadius => "x-border-radius",
            PropertyId::XBorderBeforeStartRadius => "x-border-before-start-radius",
            PropertyId::XBorderBeforeEndRadius => "x-border-before-end-radius",
            PropertyId::XBorderAfterStartRadius => "x-border-after-start-radius",
            PropertyId::XBorderAfterEndRadius => "x-border-after-end-radius",
            PropertyId::XNumberConversionFeatures => "x-number-conversion-features",
            PropertyId::XHeaderColumn => "x-header-column",
            PropertyId::XLayer => "x-layer",
            PropertyId::XAutoToggle => "x-auto-toggle",
            PropertyId::XBackgroundImageWidth => "x-background-image-width",
            PropertyId::XBackgroundImageHeight => "x-background-image-height",
            PropertyId::XAbbreviation => "x-abbreviation",
            PropertyId::Opacity => "opacity",
        }
    }

    /// Parse a property name to get the PropertyId
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "absolute-position" => Some(PropertyId::AbsolutePosition),
            "active-state" => Some(PropertyId::ActiveState),
            "alignment-adjust" => Some(PropertyId::AlignmentAdjust),
            "alignment-baseline" => Some(PropertyId::AlignmentBaseline),
            "auto-restore" => Some(PropertyId::AutoRestore),
            "azimuth" => Some(PropertyId::Azimuth),
            "background" => Some(PropertyId::Background),
            "background-attachment" => Some(PropertyId::BackgroundAttachment),
            "background-color" => Some(PropertyId::BackgroundColor),
            "background-image" => Some(PropertyId::BackgroundImage),
            "background-position" => Some(PropertyId::BackgroundPosition),
            "background-position-horizontal" => Some(PropertyId::BackgroundPositionHorizontal),
            "background-position-vertical" => Some(PropertyId::BackgroundPositionVertical),
            "background-repeat" => Some(PropertyId::BackgroundRepeat),
            "baseline-shift" => Some(PropertyId::BaselineShift),
            "blank-or-not-blank" => Some(PropertyId::BlankOrNotBlank),
            "block-progression-dimension" => Some(PropertyId::BlockProgressionDimension),
            "border" => Some(PropertyId::Border),
            "border-after-color" => Some(PropertyId::BorderAfterColor),
            "border-after-precedence" => Some(PropertyId::BorderAfterPrecedence),
            "border-after-style" => Some(PropertyId::BorderAfterStyle),
            "border-after-width" => Some(PropertyId::BorderAfterWidth),
            "border-before-color" => Some(PropertyId::BorderBeforeColor),
            "border-before-precedence" => Some(PropertyId::BorderBeforePrecedence),
            "border-before-style" => Some(PropertyId::BorderBeforeStyle),
            "border-before-width" => Some(PropertyId::BorderBeforeWidth),
            "border-bottom" => Some(PropertyId::BorderBottom),
            "border-bottom-color" => Some(PropertyId::BorderBottomColor),
            "border-bottom-style" => Some(PropertyId::BorderBottomStyle),
            "border-bottom-width" => Some(PropertyId::BorderBottomWidth),
            "border-collapse" => Some(PropertyId::BorderCollapse),
            "border-color" => Some(PropertyId::BorderColor),
            "border-end-color" => Some(PropertyId::BorderEndColor),
            "border-end-precedence" => Some(PropertyId::BorderEndPrecedence),
            "border-end-style" => Some(PropertyId::BorderEndStyle),
            "border-end-width" => Some(PropertyId::BorderEndWidth),
            "border-left" => Some(PropertyId::BorderLeft),
            "border-left-color" => Some(PropertyId::BorderLeftColor),
            "border-left-style" => Some(PropertyId::BorderLeftStyle),
            "border-left-width" => Some(PropertyId::BorderLeftWidth),
            "border-right" => Some(PropertyId::BorderRight),
            "border-right-color" => Some(PropertyId::BorderRightColor),
            "border-right-style" => Some(PropertyId::BorderRightStyle),
            "border-right-width" => Some(PropertyId::BorderRightWidth),
            "border-separation" => Some(PropertyId::BorderSeparation),
            "border-spacing" => Some(PropertyId::BorderSpacing),
            "border-start-color" => Some(PropertyId::BorderStartColor),
            "border-start-precedence" => Some(PropertyId::BorderStartPrecedence),
            "border-start-style" => Some(PropertyId::BorderStartStyle),
            "border-start-width" => Some(PropertyId::BorderStartWidth),
            "border-style" => Some(PropertyId::BorderStyle),
            "border-top" => Some(PropertyId::BorderTop),
            "border-top-color" => Some(PropertyId::BorderTopColor),
            "border-top-style" => Some(PropertyId::BorderTopStyle),
            "border-top-width" => Some(PropertyId::BorderTopWidth),
            "border-width" => Some(PropertyId::BorderWidth),
            "bottom" => Some(PropertyId::Bottom),
            "break-after" => Some(PropertyId::BreakAfter),
            "break-before" => Some(PropertyId::BreakBefore),
            "caption-side" => Some(PropertyId::CaptionSide),
            "case-name" => Some(PropertyId::CaseName),
            "case-title" => Some(PropertyId::CaseTitle),
            "change-bar-class" => Some(PropertyId::ChangeBarClass),
            "change-bar-color" => Some(PropertyId::ChangeBarColor),
            "change-bar-offset" => Some(PropertyId::ChangeBarOffset),
            "change-bar-placement" => Some(PropertyId::ChangeBarPlacement),
            "change-bar-style" => Some(PropertyId::ChangeBarStyle),
            "change-bar-width" => Some(PropertyId::ChangeBarWidth),
            "character" => Some(PropertyId::Character),
            "clear" => Some(PropertyId::Clear),
            "clip" => Some(PropertyId::Clip),
            "color" => Some(PropertyId::Color),
            "color-profile-name" => Some(PropertyId::ColorProfileName),
            "column-count" => Some(PropertyId::ColumnCount),
            "column-gap" => Some(PropertyId::ColumnGap),
            "column-number" => Some(PropertyId::ColumnNumber),
            "column-width" => Some(PropertyId::ColumnWidth),
            "content-height" => Some(PropertyId::ContentHeight),
            "content-type" => Some(PropertyId::ContentType),
            "content-width" => Some(PropertyId::ContentWidth),
            "country" => Some(PropertyId::Country),
            "cue" => Some(PropertyId::Cue),
            "cue-after" => Some(PropertyId::CueAfter),
            "cue-before" => Some(PropertyId::CueBefore),
            "destination-placement-offset" => Some(PropertyId::DestinationPlacementOffset),
            "direction" => Some(PropertyId::Direction),
            "display-align" => Some(PropertyId::DisplayAlign),
            "dominant-baseline" => Some(PropertyId::DominantBaseline),
            "elevation" => Some(PropertyId::Elevation),
            "empty-cells" => Some(PropertyId::EmptyCells),
            "end-indent" => Some(PropertyId::EndIndent),
            "ends-row" => Some(PropertyId::EndsRow),
            "extent" => Some(PropertyId::Extent),
            "external-destination" => Some(PropertyId::ExternalDestination),
            "float" => Some(PropertyId::Float),
            "flow-map-name" => Some(PropertyId::FlowMapName),
            "flow-map-reference" => Some(PropertyId::FlowMapReference),
            "flow-name" => Some(PropertyId::FlowName),
            "flow-name-reference" => Some(PropertyId::FlowNameReference),
            "font" => Some(PropertyId::Font),
            "font-family" => Some(PropertyId::FontFamily),
            "font-selection-strategy" => Some(PropertyId::FontSelectionStrategy),
            "font-size" => Some(PropertyId::FontSize),
            "font-size-adjust" => Some(PropertyId::FontSizeAdjust),
            "font-stretch" => Some(PropertyId::FontStretch),
            "font-style" => Some(PropertyId::FontStyle),
            "font-variant" => Some(PropertyId::FontVariant),
            "font-weight" => Some(PropertyId::FontWeight),
            "force-page-count" => Some(PropertyId::ForcePageCount),
            "format" => Some(PropertyId::Format),
            "glyph-orientation-horizontal" => Some(PropertyId::GlyphOrientationHorizontal),
            "glyph-orientation-vertical" => Some(PropertyId::GlyphOrientationVertical),
            "grouping-separator" => Some(PropertyId::GroupingSeparator),
            "grouping-size" => Some(PropertyId::GroupingSize),
            "height" => Some(PropertyId::Height),
            "hyphenate" => Some(PropertyId::Hyphenate),
            "hyphenation-character" => Some(PropertyId::HyphenationCharacter),
            "hyphenation-keep" => Some(PropertyId::HyphenationKeep),
            "hyphenation-ladder-count" => Some(PropertyId::HyphenationLadderCount),
            "hyphenation-push-character-count" => Some(PropertyId::HyphenationPushCharacterCount),
            "hyphenation-remain-character-count" => {
                Some(PropertyId::HyphenationRemainCharacterCount)
            }
            "id" => Some(PropertyId::Id),
            "indicate-destination" => Some(PropertyId::IndicateDestination),
            "index-class" => Some(PropertyId::IndexClass),
            "index-key" => Some(PropertyId::IndexKey),
            "initial-page-number" => Some(PropertyId::InitialPageNumber),
            "inline-progression-dimension" => Some(PropertyId::InlineProgressionDimension),
            "internal-destination" => Some(PropertyId::InternalDestination),
            "intrinsic-scale-value" => Some(PropertyId::IntrinsicScaleValue),
            "intrusion-displace" => Some(PropertyId::IntrusionDisplace),
            "keep-together" => Some(PropertyId::KeepTogether),
            "keep-with-next" => Some(PropertyId::KeepWithNext),
            "keep-with-previous" => Some(PropertyId::KeepWithPrevious),
            "language" => Some(PropertyId::Language),
            "last-line-end-indent" => Some(PropertyId::LastLineEndIndent),
            "leader-alignment" => Some(PropertyId::LeaderAlignment),
            "leader-length" => Some(PropertyId::LeaderLength),
            "leader-pattern" => Some(PropertyId::LeaderPattern),
            "leader-pattern-width" => Some(PropertyId::LeaderPatternWidth),
            "left" => Some(PropertyId::Left),
            "letter-spacing" => Some(PropertyId::LetterSpacing),
            "letter-value" => Some(PropertyId::LetterValue),
            "linefeed-treatment" => Some(PropertyId::LinefeedTreatment),
            "line-height" => Some(PropertyId::LineHeight),
            "line-height-shift-adjustment" => Some(PropertyId::LineHeightShiftAdjustment),
            "line-stacking-strategy" => Some(PropertyId::LineStackingStrategy),
            "margin" => Some(PropertyId::Margin),
            "margin-bottom" => Some(PropertyId::MarginBottom),
            "margin-left" => Some(PropertyId::MarginLeft),
            "margin-right" => Some(PropertyId::MarginRight),
            "margin-top" => Some(PropertyId::MarginTop),
            "marker-class-name" => Some(PropertyId::MarkerClassName),
            "master-name" => Some(PropertyId::MasterName),
            "master-reference" => Some(PropertyId::MasterReference),
            "max-height" => Some(PropertyId::MaxHeight),
            "maximum-repeats" => Some(PropertyId::MaximumRepeats),
            "max-width" => Some(PropertyId::MaxWidth),
            "merge-pages-across-index-key-references" => {
                Some(PropertyId::MergePagesAcrossIndexKeyReferences)
            }
            "merge-ranges-across-index-key-references" => {
                Some(PropertyId::MergeRangesAcrossIndexKeyReferences)
            }
            "merge-sequential-page-numbers" => Some(PropertyId::MergeSequentialPageNumbers),
            "media-usage" => Some(PropertyId::MediaUsage),
            "min-height" => Some(PropertyId::MinHeight),
            "min-width" => Some(PropertyId::MinWidth),
            "number-columns-repeated" => Some(PropertyId::NumberColumnsRepeated),
            "number-columns-spanned" => Some(PropertyId::NumberColumnsSpanned),
            "number-rows-spanned" => Some(PropertyId::NumberRowsSpanned),
            "odd-or-even" => Some(PropertyId::OddOrEven),
            "orphans" => Some(PropertyId::Orphans),
            "overflow" => Some(PropertyId::Overflow),
            "padding" => Some(PropertyId::Padding),
            "padding-after" => Some(PropertyId::PaddingAfter),
            "padding-before" => Some(PropertyId::PaddingBefore),
            "padding-bottom" => Some(PropertyId::PaddingBottom),
            "padding-end" => Some(PropertyId::PaddingEnd),
            "padding-left" => Some(PropertyId::PaddingLeft),
            "padding-right" => Some(PropertyId::PaddingRight),
            "padding-start" => Some(PropertyId::PaddingStart),
            "padding-top" => Some(PropertyId::PaddingTop),
            "page-break-after" => Some(PropertyId::PageBreakAfter),
            "page-break-before" => Some(PropertyId::PageBreakBefore),
            "page-break-inside" => Some(PropertyId::PageBreakInside),
            "page-citation-strategy" => Some(PropertyId::PageCitationStrategy),
            "page-height" => Some(PropertyId::PageHeight),
            "page-number-treatment" => Some(PropertyId::PageNumberTreatment),
            "page-position" => Some(PropertyId::PagePosition),
            "page-width" => Some(PropertyId::PageWidth),
            "pause" => Some(PropertyId::Pause),
            "pause-after" => Some(PropertyId::PauseAfter),
            "pause-before" => Some(PropertyId::PauseBefore),
            "pitch" => Some(PropertyId::Pitch),
            "pitch-range" => Some(PropertyId::PitchRange),
            "play-during" => Some(PropertyId::PlayDuring),
            "position" => Some(PropertyId::Position),
            "precedence" => Some(PropertyId::Precedence),
            "provisional-distance-between-starts" => {
                Some(PropertyId::ProvisionalDistanceBetweenStarts)
            }
            "provisional-label-separation" => Some(PropertyId::ProvisionalLabelSeparation),
            "reference-orientation" => Some(PropertyId::ReferenceOrientation),
            "ref-id" => Some(PropertyId::RefId),
            "region-name" => Some(PropertyId::RegionName),
            "region-name-reference" => Some(PropertyId::RegionNameReference),
            "ref-index-key" => Some(PropertyId::RefIndexKey),
            "relative-align" => Some(PropertyId::RelativeAlign),
            "relative-position" => Some(PropertyId::RelativePosition),
            "rendering-intent" => Some(PropertyId::RenderingIntent),
            "retrieve-boundary" => Some(PropertyId::RetrieveBoundary),
            "retrieve-boundary-within-table" => Some(PropertyId::RetrieveBoundaryWithinTable),
            "retrieve-class-name" => Some(PropertyId::RetrieveClassName),
            "retrieve-position" => Some(PropertyId::RetrievePosition),
            "retrieve-position-within-table" => Some(PropertyId::RetrievePositionWithinTable),
            "richness" => Some(PropertyId::Richness),
            "right" => Some(PropertyId::Right),
            "role" => Some(PropertyId::Role),
            "rule-style" => Some(PropertyId::RuleStyle),
            "rule-thickness" => Some(PropertyId::RuleThickness),
            "scaling" => Some(PropertyId::Scaling),
            "scaling-method" => Some(PropertyId::ScalingMethod),
            "score-spaces" => Some(PropertyId::ScoreSpaces),
            "script" => Some(PropertyId::Script),
            "show-destination" => Some(PropertyId::ShowDestination),
            "size" => Some(PropertyId::Size),
            "source-document" => Some(PropertyId::SourceDocument),
            "space-after" => Some(PropertyId::SpaceAfter),
            "space-before" => Some(PropertyId::SpaceBefore),
            "space-end" => Some(PropertyId::SpaceEnd),
            "space-start" => Some(PropertyId::SpaceStart),
            "span" => Some(PropertyId::Span),
            "speak" => Some(PropertyId::Speak),
            "speak-header" => Some(PropertyId::SpeakHeader),
            "speak-numeral" => Some(PropertyId::SpeakNumeral),
            "speak-punctuation" => Some(PropertyId::SpeakPunctuation),
            "speech-rate" => Some(PropertyId::SpeechRate),
            "src" => Some(PropertyId::Src),
            "start-indent" => Some(PropertyId::StartIndent),
            "starting-state" => Some(PropertyId::StartingState),
            "starts-row" => Some(PropertyId::StartsRow),
            "stress" => Some(PropertyId::Stress),
            "suppress-at-line-break" => Some(PropertyId::SuppressAtLineBreak),
            "switch-to" => Some(PropertyId::SwitchTo),
            "table-layout" => Some(PropertyId::TableLayout),
            "table-omit-footer-at-break" => Some(PropertyId::TableOmitFooterAtBreak),
            "table-omit-header-at-break" => Some(PropertyId::TableOmitHeaderAtBreak),
            "target-presentation-context" => Some(PropertyId::TargetPresentationContext),
            "target-processing-context" => Some(PropertyId::TargetProcessingContext),
            "target-stylesheet" => Some(PropertyId::TargetStylesheet),
            "text-align" => Some(PropertyId::TextAlign),
            "text-align-last" => Some(PropertyId::TextAlignLast),
            "text-altitude" => Some(PropertyId::TextAltitude),
            "text-decoration" => Some(PropertyId::TextDecoration),
            "text-depth" => Some(PropertyId::TextDepth),
            "text-indent" => Some(PropertyId::TextIndent),
            "text-shadow" => Some(PropertyId::TextShadow),
            "text-transform" => Some(PropertyId::TextTransform),
            "top" => Some(PropertyId::Top),
            "treat-as-word-space" => Some(PropertyId::TreatAsWordSpace),
            "unicode-bidi" => Some(PropertyId::UnicodeBidi),
            "vertical-align" => Some(PropertyId::VerticalAlign),
            "visibility" => Some(PropertyId::Visibility),
            "voice-family" => Some(PropertyId::VoiceFamily),
            "volume" => Some(PropertyId::Volume),
            "white-space" => Some(PropertyId::WhiteSpace),
            "white-space-collapse" => Some(PropertyId::WhiteSpaceCollapse),
            "white-space-treatment" => Some(PropertyId::WhiteSpaceTreatment),
            "widows" => Some(PropertyId::Widows),
            "width" => Some(PropertyId::Width),
            "word-spacing" => Some(PropertyId::WordSpacing),
            "wrap-option" => Some(PropertyId::WrapOption),
            "writing-mode" => Some(PropertyId::WritingMode),
            "xml-lang" | "xml:lang" => Some(PropertyId::XmlLang),
            "z-index" => Some(PropertyId::ZIndex),
            "x-widow-content-limit" => Some(PropertyId::XWidowContentLimit),
            "x-orphan-content-limit" => Some(PropertyId::XOrphanContentLimit),
            "x-disable-column-balancing" => Some(PropertyId::XDisableColumnBalancing),
            "x-alt-text" => Some(PropertyId::XAltText),
            "x-xml-base" => Some(PropertyId::XXmlBase),
            "x-border-before-radius-start" => Some(PropertyId::XBorderBeforeRadiusStart),
            "x-border-before-radius-end" => Some(PropertyId::XBorderBeforeRadiusEnd),
            "x-border-after-radius-start" => Some(PropertyId::XBorderAfterRadiusStart),
            "x-border-after-radius-end" => Some(PropertyId::XBorderAfterRadiusEnd),
            "x-border-start-radius-before" => Some(PropertyId::XBorderStartRadiusBefore),
            "x-border-start-radius-after" => Some(PropertyId::XBorderStartRadiusAfter),
            "x-border-end-radius-before" => Some(PropertyId::XBorderEndRadiusBefore),
            "x-border-end-radius-after" => Some(PropertyId::XBorderEndRadiusAfter),
            "x-border-radius" => Some(PropertyId::XBorderRadius),
            "x-border-before-start-radius" => Some(PropertyId::XBorderBeforeStartRadius),
            "x-border-before-end-radius" => Some(PropertyId::XBorderBeforeEndRadius),
            "x-border-after-start-radius" => Some(PropertyId::XBorderAfterStartRadius),
            "x-border-after-end-radius" => Some(PropertyId::XBorderAfterEndRadius),
            "x-number-conversion-features" => Some(PropertyId::XNumberConversionFeatures),
            "x-header-column" => Some(PropertyId::XHeaderColumn),
            "x-layer" => Some(PropertyId::XLayer),
            "x-auto-toggle" => Some(PropertyId::XAutoToggle),
            "x-background-image-width" => Some(PropertyId::XBackgroundImageWidth),
            "x-background-image-height" => Some(PropertyId::XBackgroundImageHeight),
            "x-abbreviation" => Some(PropertyId::XAbbreviation),
            "opacity" => Some(PropertyId::Opacity),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== PropertyId::name() tests =====

    #[test]
    fn test_name_font_size() {
        assert_eq!(PropertyId::FontSize.name(), "font-size");
    }

    #[test]
    fn test_name_font_family() {
        assert_eq!(PropertyId::FontFamily.name(), "font-family");
    }

    #[test]
    fn test_name_font_weight() {
        assert_eq!(PropertyId::FontWeight.name(), "font-weight");
    }

    #[test]
    fn test_name_font_style() {
        assert_eq!(PropertyId::FontStyle.name(), "font-style");
    }

    #[test]
    fn test_name_color() {
        assert_eq!(PropertyId::Color.name(), "color");
    }

    #[test]
    fn test_name_background_color() {
        assert_eq!(PropertyId::BackgroundColor.name(), "background-color");
    }

    #[test]
    fn test_name_margin_top() {
        assert_eq!(PropertyId::MarginTop.name(), "margin-top");
    }

    #[test]
    fn test_name_margin_left() {
        assert_eq!(PropertyId::MarginLeft.name(), "margin-left");
    }

    #[test]
    fn test_name_margin_right() {
        assert_eq!(PropertyId::MarginRight.name(), "margin-right");
    }

    #[test]
    fn test_name_margin_bottom() {
        assert_eq!(PropertyId::MarginBottom.name(), "margin-bottom");
    }

    #[test]
    fn test_name_padding_top() {
        assert_eq!(PropertyId::PaddingTop.name(), "padding-top");
    }

    #[test]
    fn test_name_border_top_width() {
        assert_eq!(PropertyId::BorderTopWidth.name(), "border-top-width");
    }

    #[test]
    fn test_name_border_top_style() {
        assert_eq!(PropertyId::BorderTopStyle.name(), "border-top-style");
    }

    #[test]
    fn test_name_border_top_color() {
        assert_eq!(PropertyId::BorderTopColor.name(), "border-top-color");
    }

    #[test]
    fn test_name_text_align() {
        assert_eq!(PropertyId::TextAlign.name(), "text-align");
    }

    #[test]
    fn test_name_text_indent() {
        assert_eq!(PropertyId::TextIndent.name(), "text-indent");
    }

    #[test]
    fn test_name_line_height() {
        assert_eq!(PropertyId::LineHeight.name(), "line-height");
    }

    #[test]
    fn test_name_width() {
        assert_eq!(PropertyId::Width.name(), "width");
    }

    #[test]
    fn test_name_height() {
        assert_eq!(PropertyId::Height.name(), "height");
    }

    #[test]
    fn test_name_opacity() {
        assert_eq!(PropertyId::Opacity.name(), "opacity");
    }

    #[test]
    fn test_name_overflow() {
        assert_eq!(PropertyId::Overflow.name(), "overflow");
    }

    #[test]
    fn test_name_visibility() {
        assert_eq!(PropertyId::Visibility.name(), "visibility");
    }

    #[test]
    fn test_name_display_align() {
        assert_eq!(PropertyId::DisplayAlign.name(), "display-align");
    }

    #[test]
    fn test_name_position() {
        assert_eq!(PropertyId::Position.name(), "position");
    }

    #[test]
    fn test_name_z_index() {
        assert_eq!(PropertyId::ZIndex.name(), "z-index");
    }

    #[test]
    fn test_name_direction() {
        assert_eq!(PropertyId::Direction.name(), "direction");
    }

    #[test]
    fn test_name_writing_mode() {
        assert_eq!(PropertyId::WritingMode.name(), "writing-mode");
    }

    #[test]
    fn test_name_column_count() {
        assert_eq!(PropertyId::ColumnCount.name(), "column-count");
    }

    #[test]
    fn test_name_break_before() {
        assert_eq!(PropertyId::BreakBefore.name(), "break-before");
    }

    #[test]
    fn test_name_break_after() {
        assert_eq!(PropertyId::BreakAfter.name(), "break-after");
    }

    #[test]
    fn test_name_keep_with_next() {
        assert_eq!(PropertyId::KeepWithNext.name(), "keep-with-next");
    }

    #[test]
    fn test_name_keep_with_previous() {
        assert_eq!(PropertyId::KeepWithPrevious.name(), "keep-with-previous");
    }

    #[test]
    fn test_name_orphans() {
        assert_eq!(PropertyId::Orphans.name(), "orphans");
    }

    #[test]
    fn test_name_widows() {
        assert_eq!(PropertyId::Widows.name(), "widows");
    }

    #[test]
    fn test_name_hyphenate() {
        assert_eq!(PropertyId::Hyphenate.name(), "hyphenate");
    }

    #[test]
    fn test_name_xml_lang() {
        assert_eq!(PropertyId::XmlLang.name(), "xml-lang");
    }

    #[test]
    fn test_name_border_collapse() {
        assert_eq!(PropertyId::BorderCollapse.name(), "border-collapse");
    }

    #[test]
    fn test_name_absolute_position() {
        assert_eq!(PropertyId::AbsolutePosition.name(), "absolute-position");
    }

    // ===== PropertyId::from_name() tests =====

    #[test]
    fn test_from_name_font_size() {
        assert_eq!(
            PropertyId::from_name("font-size"),
            Some(PropertyId::FontSize)
        );
    }

    #[test]
    fn test_from_name_font_family() {
        assert_eq!(
            PropertyId::from_name("font-family"),
            Some(PropertyId::FontFamily)
        );
    }

    #[test]
    fn test_from_name_color() {
        assert_eq!(PropertyId::from_name("color"), Some(PropertyId::Color));
    }

    #[test]
    fn test_from_name_background_color() {
        assert_eq!(
            PropertyId::from_name("background-color"),
            Some(PropertyId::BackgroundColor)
        );
    }

    #[test]
    fn test_from_name_margin_top() {
        assert_eq!(
            PropertyId::from_name("margin-top"),
            Some(PropertyId::MarginTop)
        );
    }

    #[test]
    fn test_from_name_padding_left() {
        assert_eq!(
            PropertyId::from_name("padding-left"),
            Some(PropertyId::PaddingLeft)
        );
    }

    #[test]
    fn test_from_name_text_align() {
        assert_eq!(
            PropertyId::from_name("text-align"),
            Some(PropertyId::TextAlign)
        );
    }

    #[test]
    fn test_from_name_line_height() {
        assert_eq!(
            PropertyId::from_name("line-height"),
            Some(PropertyId::LineHeight)
        );
    }

    #[test]
    fn test_from_name_width() {
        assert_eq!(PropertyId::from_name("width"), Some(PropertyId::Width));
    }

    #[test]
    fn test_from_name_height() {
        assert_eq!(PropertyId::from_name("height"), Some(PropertyId::Height));
    }

    #[test]
    fn test_from_name_opacity() {
        assert_eq!(PropertyId::from_name("opacity"), Some(PropertyId::Opacity));
    }

    #[test]
    fn test_from_name_overflow() {
        assert_eq!(
            PropertyId::from_name("overflow"),
            Some(PropertyId::Overflow)
        );
    }

    #[test]
    fn test_from_name_unknown_returns_none() {
        assert_eq!(PropertyId::from_name("not-a-property"), None);
    }

    #[test]
    fn test_from_name_empty_returns_none() {
        assert_eq!(PropertyId::from_name(""), None);
    }

    #[test]
    fn test_from_name_case_sensitive() {
        // Property names are case-sensitive: "Font-Size" should NOT parse
        assert_eq!(PropertyId::from_name("Font-Size"), None);
        assert_eq!(PropertyId::from_name("FONT-SIZE"), None);
    }

    // ===== xml:lang special alias tests =====

    #[test]
    fn test_from_name_xml_lang_hyphen_alias() {
        // "xml-lang" is the canonical name
        assert_eq!(PropertyId::from_name("xml-lang"), Some(PropertyId::XmlLang));
    }

    #[test]
    fn test_from_name_xml_lang_colon_alias() {
        // "xml:lang" should also be recognized as an alias
        assert_eq!(PropertyId::from_name("xml:lang"), Some(PropertyId::XmlLang));
    }

    // ===== Round-trip tests: name() then from_name() =====

    #[test]
    fn test_roundtrip_font_size() {
        let id = PropertyId::FontSize;
        assert_eq!(PropertyId::from_name(id.name()), Some(id));
    }

    #[test]
    fn test_roundtrip_color() {
        let id = PropertyId::Color;
        assert_eq!(PropertyId::from_name(id.name()), Some(id));
    }

    #[test]
    fn test_roundtrip_margin_top() {
        let id = PropertyId::MarginTop;
        assert_eq!(PropertyId::from_name(id.name()), Some(id));
    }

    #[test]
    fn test_roundtrip_border_top_style() {
        let id = PropertyId::BorderTopStyle;
        assert_eq!(PropertyId::from_name(id.name()), Some(id));
    }

    #[test]
    fn test_roundtrip_text_align() {
        let id = PropertyId::TextAlign;
        assert_eq!(PropertyId::from_name(id.name()), Some(id));
    }

    #[test]
    fn test_roundtrip_opacity() {
        let id = PropertyId::Opacity;
        assert_eq!(PropertyId::from_name(id.name()), Some(id));
    }

    #[test]
    fn test_roundtrip_writing_mode() {
        let id = PropertyId::WritingMode;
        assert_eq!(PropertyId::from_name(id.name()), Some(id));
    }

    #[test]
    fn test_roundtrip_break_before() {
        let id = PropertyId::BreakBefore;
        assert_eq!(PropertyId::from_name(id.name()), Some(id));
    }

    #[test]
    fn test_roundtrip_keep_with_next() {
        let id = PropertyId::KeepWithNext;
        assert_eq!(PropertyId::from_name(id.name()), Some(id));
    }

    #[test]
    fn test_roundtrip_hyphenate() {
        let id = PropertyId::Hyphenate;
        assert_eq!(PropertyId::from_name(id.name()), Some(id));
    }

    // ===== Discriminant value tests =====

    #[test]
    fn test_discriminant_values_are_unique() {
        // A representative sample of properties should have unique discriminant values
        let ids = [
            PropertyId::FontSize,
            PropertyId::FontFamily,
            PropertyId::Color,
            PropertyId::MarginTop,
            PropertyId::MarginLeft,
            PropertyId::PaddingTop,
            PropertyId::TextAlign,
            PropertyId::LineHeight,
            PropertyId::Width,
            PropertyId::Height,
            PropertyId::Opacity,
        ];

        let mut seen = std::collections::HashSet::new();
        for id in &ids {
            let discriminant = *id as u16;
            assert!(
                seen.insert(discriminant),
                "Duplicate discriminant {} for {:?}",
                discriminant,
                id
            );
        }
    }

    #[test]
    fn test_absolute_position_discriminant_is_1() {
        assert_eq!(PropertyId::AbsolutePosition as u16, 1);
    }

    #[test]
    fn test_opacity_is_last_or_high() {
        // Opacity should be a valid property ID
        let discriminant = PropertyId::Opacity as u16;
        assert!(discriminant >= 1, "Opacity discriminant must be >= 1");
        assert!(discriminant <= 295, "Opacity discriminant must be <= 295");
    }

    // ===== Debug/Clone/Copy traits test =====

    #[test]
    fn test_copy_semantics() {
        let id = PropertyId::FontSize;
        let copied = id;
        assert_eq!(id, copied);
    }

    #[test]
    fn test_clone_semantics() {
        let id = PropertyId::Color;
        let cloned = id;
        assert_eq!(id, cloned);
    }

    #[test]
    fn test_equality() {
        assert_eq!(PropertyId::FontSize, PropertyId::FontSize);
        assert_ne!(PropertyId::FontSize, PropertyId::FontFamily);
    }

    #[test]
    fn test_hash_consistency() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(PropertyId::FontSize, "font-size");
        map.insert(PropertyId::Color, "color");

        assert_eq!(map.get(&PropertyId::FontSize), Some(&"font-size"));
        assert_eq!(map.get(&PropertyId::Color), Some(&"color"));
        assert_eq!(map.get(&PropertyId::Opacity), None);
    }

    #[test]
    fn test_debug_format() {
        let debug_str = format!("{:?}", PropertyId::FontSize);
        assert!(!debug_str.is_empty());
        // Debug format should contain some representation
        assert!(debug_str.contains("FontSize") || !debug_str.is_empty());
    }

    // ===== Comprehensive from_name tests for property families =====

    #[test]
    fn test_from_name_all_margin_sides() {
        assert_eq!(
            PropertyId::from_name("margin-top"),
            Some(PropertyId::MarginTop)
        );
        assert_eq!(
            PropertyId::from_name("margin-right"),
            Some(PropertyId::MarginRight)
        );
        assert_eq!(
            PropertyId::from_name("margin-bottom"),
            Some(PropertyId::MarginBottom)
        );
        assert_eq!(
            PropertyId::from_name("margin-left"),
            Some(PropertyId::MarginLeft)
        );
    }

    #[test]
    fn test_from_name_all_padding_sides() {
        assert_eq!(
            PropertyId::from_name("padding-top"),
            Some(PropertyId::PaddingTop)
        );
        assert_eq!(
            PropertyId::from_name("padding-right"),
            Some(PropertyId::PaddingRight)
        );
        assert_eq!(
            PropertyId::from_name("padding-bottom"),
            Some(PropertyId::PaddingBottom)
        );
        assert_eq!(
            PropertyId::from_name("padding-left"),
            Some(PropertyId::PaddingLeft)
        );
    }

    #[test]
    fn test_from_name_border_top_sides() {
        assert_eq!(
            PropertyId::from_name("border-top-width"),
            Some(PropertyId::BorderTopWidth)
        );
        assert_eq!(
            PropertyId::from_name("border-top-style"),
            Some(PropertyId::BorderTopStyle)
        );
        assert_eq!(
            PropertyId::from_name("border-top-color"),
            Some(PropertyId::BorderTopColor)
        );
    }

    #[test]
    fn test_from_name_border_logical_sides() {
        assert_eq!(
            PropertyId::from_name("border-before-style"),
            Some(PropertyId::BorderBeforeStyle)
        );
        assert_eq!(
            PropertyId::from_name("border-after-style"),
            Some(PropertyId::BorderAfterStyle)
        );
        assert_eq!(
            PropertyId::from_name("border-start-style"),
            Some(PropertyId::BorderStartStyle)
        );
        assert_eq!(
            PropertyId::from_name("border-end-style"),
            Some(PropertyId::BorderEndStyle)
        );
    }

    #[test]
    fn test_from_name_break_properties() {
        assert_eq!(
            PropertyId::from_name("break-before"),
            Some(PropertyId::BreakBefore)
        );
        assert_eq!(
            PropertyId::from_name("break-after"),
            Some(PropertyId::BreakAfter)
        );
    }

    #[test]
    fn test_from_name_position_properties() {
        assert_eq!(PropertyId::from_name("top"), Some(PropertyId::Top));
        assert_eq!(PropertyId::from_name("bottom"), Some(PropertyId::Bottom));
        assert_eq!(PropertyId::from_name("left"), Some(PropertyId::Left));
        assert_eq!(PropertyId::from_name("right"), Some(PropertyId::Right));
    }

    #[test]
    fn test_from_name_font_properties() {
        assert_eq!(
            PropertyId::from_name("font-size"),
            Some(PropertyId::FontSize)
        );
        assert_eq!(
            PropertyId::from_name("font-weight"),
            Some(PropertyId::FontWeight)
        );
        assert_eq!(
            PropertyId::from_name("font-style"),
            Some(PropertyId::FontStyle)
        );
        assert_eq!(
            PropertyId::from_name("font-family"),
            Some(PropertyId::FontFamily)
        );
        assert_eq!(
            PropertyId::from_name("font-variant"),
            Some(PropertyId::FontVariant)
        );
    }

    #[test]
    fn test_from_name_table_properties() {
        assert_eq!(
            PropertyId::from_name("border-collapse"),
            Some(PropertyId::BorderCollapse)
        );
        assert_eq!(
            PropertyId::from_name("table-layout"),
            Some(PropertyId::TableLayout)
        );
        assert_eq!(
            PropertyId::from_name("empty-cells"),
            Some(PropertyId::EmptyCells)
        );
    }

    #[test]
    fn test_from_name_hyphenation_properties() {
        assert_eq!(
            PropertyId::from_name("hyphenate"),
            Some(PropertyId::Hyphenate)
        );
        assert_eq!(
            PropertyId::from_name("hyphenation-character"),
            Some(PropertyId::HyphenationCharacter)
        );
        assert_eq!(
            PropertyId::from_name("hyphenation-push-character-count"),
            Some(PropertyId::HyphenationPushCharacterCount)
        );
        assert_eq!(
            PropertyId::from_name("hyphenation-remain-character-count"),
            Some(PropertyId::HyphenationRemainCharacterCount)
        );
    }

    // ===== name() completeness spot check =====

    #[test]
    fn test_name_is_nonempty_for_key_properties() {
        let properties = [
            PropertyId::AbsolutePosition,
            PropertyId::Color,
            PropertyId::FontSize,
            PropertyId::FontFamily,
            PropertyId::FontWeight,
            PropertyId::FontStyle,
            PropertyId::LineHeight,
            PropertyId::TextAlign,
            PropertyId::TextIndent,
            PropertyId::MarginTop,
            PropertyId::PaddingLeft,
            PropertyId::BorderTopStyle,
            PropertyId::Width,
            PropertyId::Height,
            PropertyId::Opacity,
            PropertyId::Overflow,
            PropertyId::Visibility,
            PropertyId::Direction,
            PropertyId::WritingMode,
            PropertyId::Orphans,
            PropertyId::Widows,
            PropertyId::BreakBefore,
            PropertyId::BreakAfter,
            PropertyId::Hyphenate,
            PropertyId::XmlLang,
        ];

        for prop in &properties {
            let name = prop.name();
            assert!(!name.is_empty(), "Property {:?} has empty name", prop);
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '-' || c == ':'),
                "Property {:?} name '{}' contains unexpected characters",
                prop,
                name
            );
        }
    }
}
