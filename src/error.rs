use std::{
    error::Error,
    fmt::{Display, Formatter},
    io,
};

use crate::aesthetics::{Aesthetic, AestheticDomain, AestheticProperty};
use crate::data::VectorType;

pub type Result<T> = std::result::Result<T, PlotError>;

/// Describes expected or actual data types in error messages
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    /// A vector type (int, float, string)
    Vector(VectorType),
    /// A numeric type (int or float)
    Numeric,
    /// A constant value of a specific type
    Constant(VectorType),
    /// An RGBA integer constant for colors
    RgbaConstant,
    /// A column mapping
    ColumnMapping,
    /// A categorical scale
    CategoricalScale,
    /// A custom description
    Custom(String),
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Vector(vtype) => write!(f, "{}", vtype),
            DataType::Numeric => write!(f, "numeric"),
            DataType::Constant(vtype) => write!(f, "{} constant", vtype),
            DataType::RgbaConstant => write!(f, "RGBA integer constant"),
            DataType::ColumnMapping => write!(f, "column mapping"),
            DataType::CategoricalScale => write!(f, "categorical scale"),
            DataType::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug)]
pub enum PlotError {
    /// A required aesthetic is missing from the mapping
    MissingAesthetic {
        aesthetic: Aesthetic,
    },

    /// A column referenced in a mapping is missing from the data
    MissingColumn {
        column: String,
    },

    /// An aesthetic has an invalid type (e.g., expected float but got string)
    InvalidAestheticType {
        aesthetic: Aesthetic,
        expected: DataType,
        actual: DataType,
    },

    /// A column has an invalid type for the operation
    InvalidColumnType {
        column: String,
        expected: DataType,
    },

    /// Scale configuration error (e.g., mismatched breaks and labels)
    ScaleMismatch {
        breaks_count: usize,
        labels_count: usize,
    },

    /// Invalid scale limits
    InvalidLimits {
        min: f64,
        max: f64,
    },

    /// A required stat input is missing
    MissingStatInput {
        stat: String,
        aesthetic: Aesthetic,
    },

    /// No valid data for statistical transformation
    NoValidData {
        reason: String,
    },

    /// Data source is missing
    NoDataSource,

    /// File I/O error
    IoError {
        operation: String,
        source: io::Error,
    },

    /// Cairo rendering error
    RenderError {
        operation: String,
        message: String,
    },

    /// Invalid file path
    InvalidPath {
        path: String,
    },

    /// Unsupported file format
    UnsupportedFormat {
        extension: String,
    },

    /// Invalid conversion of aesthetic value to primary aesthetic
    InvalidAestheticConversion {
        from: Aesthetic,
    },

    /// Aesthetic has conflicting scale type requirements
    AestheticTypeMismatch {
        aesthetic: Aesthetic,
        user_hint: String,
        geom_requirement: String,
        reason: String,
    },

    /// Aesthetic requires discrete data but continuous was provided
    /// or vice versa.
    AestheticDomainMismatch {
        expected: AestheticDomain,
        actual: DataType,
    },

    /// String column requires categorical scale but continuous was requested
    StringColumnRequiresCategorical {
        aesthetic: Aesthetic,
        column: String,
    },

    StatError {
        stat: String,
        details: String,
    },

    /// A required aesthetic property is missing from the layer
    MissingRequiredAesthetic {
        property: AestheticProperty,
    },

    /// Aesthetic domain conflicts - same property mapped with different domains
    AestheticDomainConflict {
        property: AestheticProperty,
        domain1: AestheticDomain,
        domain2: AestheticDomain,
    },

    /// Aesthetic domain incompatible with geom requirements
    IncompatibleDomain {
        property: AestheticProperty,
        required: AestheticDomain,
        actual: AestheticDomain,
    },

    Other {
        details: String,
    },
}

impl Display for PlotError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlotError::MissingAesthetic { aesthetic } => {
                write!(f, "Missing required aesthetic: {:?}", aesthetic)
            }
            PlotError::MissingColumn { column } => {
                write!(f, "Column '{}' not found in data", column)
            }
            PlotError::InvalidAestheticType {
                aesthetic,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Invalid type for aesthetic {:?}: expected {}, got {}",
                    aesthetic, expected, actual
                )
            }
            PlotError::InvalidColumnType { column, expected } => {
                write!(
                    f,
                    "Column '{}' has invalid type: expected {}",
                    column, expected
                )
            }
            PlotError::ScaleMismatch {
                breaks_count,
                labels_count,
            } => {
                write!(
                    f,
                    "Scale breaks and labels have mismatched lengths: {} breaks, {} labels",
                    breaks_count, labels_count
                )
            }
            PlotError::InvalidLimits { min, max } => {
                write!(f, "Invalid scale limits: min={}, max={}", min, max)
            }
            PlotError::MissingStatInput { stat, aesthetic } => {
                write!(f, "{} stat requires {:?} aesthetic", stat, aesthetic)
            }
            PlotError::NoValidData { reason } => {
                write!(f, "No valid data for operation: {}", reason)
            }
            PlotError::NoDataSource => {
                write!(f, "No data source provided")
            }
            PlotError::IoError { operation, source } => {
                write!(f, "I/O error during {}: {}", operation, source)
            }
            PlotError::RenderError { operation, message } => {
                write!(f, "Render error during {}: {}", operation, message)
            }
            PlotError::InvalidPath { path } => {
                write!(f, "Invalid file path: {}", path)
            }
            PlotError::UnsupportedFormat { extension } => {
                write!(f, "Unsupported file format: {}", extension)
            }
            PlotError::InvalidAestheticConversion { from } => {
                write!(
                    f,
                    "Cannot convert aesthetic {:?} to primary aesthetic",
                    from
                )
            }
            PlotError::AestheticTypeMismatch {
                aesthetic,
                user_hint,
                geom_requirement,
                reason,
            } => {
                write!(
                    f,
                    "Scale type conflict for {:?}: user specified {}, geom requires {}. {}",
                    aesthetic, user_hint, geom_requirement, reason
                )
            }
            PlotError::AestheticDomainMismatch { expected, actual } => {
                write!(
                    f,
                    "Aesthetic domain mismatch: expected {:?}, got type {}",
                    expected, actual
                )
            }
            PlotError::StringColumnRequiresCategorical { aesthetic, column } => {
                write!(
                    f,
                    "Column '{}' is a string and must use categorical scale for {:?}, but continuous was requested",
                    column, aesthetic
                )
            }
            PlotError::StatError { stat, details } => {
                write!(f, "Stat '{}' error: {}", stat, details)
            }
            PlotError::MissingRequiredAesthetic { property } => {
                write!(f, "Missing required aesthetic property: {:?}", property)
            }
            PlotError::AestheticDomainConflict { property, domain1, domain2 } => {
                write!(
                    f,
                    "Aesthetic property {:?} has conflicting domains: {:?} vs {:?}",
                    property, domain1, domain2
                )
            }
            PlotError::IncompatibleDomain { property, required, actual } => {
                write!(
                    f,
                    "Aesthetic property {:?} requires {:?} domain, but {:?} was provided",
                    property, required, actual
                )
            }
            PlotError::Other { details } => {
                write!(f, "Plot error: {}", details)
            }
        }
    }
}

impl Error for PlotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PlotError::IoError { source, .. } => Some(source),
            _ => None,
        }
    }
}

// Convenience constructors
impl PlotError {
    pub fn missing_column(column: impl Into<String>) -> Self {
        PlotError::MissingColumn {
            column: column.into(),
        }
    }

    pub fn invalid_column_type(column: impl Into<String>, expected: impl Into<DataType>) -> Self {
        PlotError::InvalidColumnType {
            column: column.into(),
            expected: expected.into(),
        }
    }

    pub fn missing_stat_input(stat: impl Into<String>, aesthetic: Aesthetic) -> Self {
        PlotError::MissingStatInput {
            stat: stat.into(),
            aesthetic,
        }
    }

    pub fn no_valid_data(reason: impl Into<String>) -> Self {
        PlotError::NoValidData {
            reason: reason.into(),
        }
    }

    pub fn io_error(operation: impl Into<String>, source: io::Error) -> Self {
        PlotError::IoError {
            operation: operation.into(),
            source,
        }
    }

    pub fn render_error(operation: impl Into<String>, message: impl Into<String>) -> Self {
        PlotError::RenderError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    pub fn invalid_path(path: impl Into<String>) -> Self {
        PlotError::InvalidPath { path: path.into() }
    }

    pub fn unsupported_format(extension: impl Into<String>) -> Self {
        PlotError::UnsupportedFormat {
            extension: extension.into(),
        }
    }

    pub fn aesthetic_type_mismatch(
        aesthetic: Aesthetic,
        user_hint: impl Into<String>,
        geom_requirement: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        PlotError::AestheticTypeMismatch {
            aesthetic,
            user_hint: user_hint.into(),
            geom_requirement: geom_requirement.into(),
            reason: reason.into(),
        }
    }

    pub fn string_column_requires_categorical(
        aesthetic: Aesthetic,
        column: impl Into<String>,
    ) -> Self {
        PlotError::StringColumnRequiresCategorical {
            aesthetic,
            column: column.into(),
        }
    }
}

// Conversions for ergonomic error creation
impl From<&str> for DataType {
    fn from(s: &str) -> Self {
        match s {
            "numeric" => DataType::Numeric,
            "int" | "integer" => DataType::Vector(VectorType::Int),
            "float" => DataType::Vector(VectorType::Float),
            "str" | "string" => DataType::Vector(VectorType::Str),
            _ => DataType::Custom(s.to_string()),
        }
    }
}

impl From<String> for DataType {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<VectorType> for DataType {
    fn from(vtype: VectorType) -> Self {
        DataType::Vector(vtype)
    }
}

/// Helper function to convert Cairo errors to PlotError
///
/// # Example
/// ```ignore
/// ctx.cairo.stroke().map_err(to_plot_error)?;
/// ```
pub fn to_plot_error(err: cairo::Error) -> PlotError {
    PlotError::render_error("cairo operation", format!("{}", err))
}

pub fn to_io_error(err: PlotError) -> io::Error {
    match err {
        PlotError::IoError { source, .. } => source,
        _ => io::Error::new(io::ErrorKind::Other, err),
    }
}