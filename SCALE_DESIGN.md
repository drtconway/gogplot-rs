# Scale Design for Rust ggplot2

## Problem
Scales need to handle different input/output types:
- Domain: continuous (f64) vs discrete (categorical strings/ints)
- Range: continuous (position, size, alpha → f64) vs discrete (color, shape → enum types)

## Approach 1: Type Parameters (Strongly Typed)
```rust
pub trait Scale<Input, Output> {
    fn train(&mut self, data: &[Input]);
    fn map_value(&self, value: Input) -> Option<Output>;
    fn inverse(&self, value: Output) -> Input;
}

// Examples:
// Scale<f64, f64> - continuous → continuous (position, size, alpha)
// Scale<f64, Color> - continuous → discrete (gradient colormaps)
// Scale<String, Color> - discrete → discrete (categorical colors)
// Scale<String, PointShape> - discrete → discrete (categorical shapes)
```

**Pros:**
- Type safe at compile time
- No runtime overhead
- Clear about what each scale does

**Cons:**
- Makes ScaleSet complex (needs separate fields for each type)
- Trait objects become harder (`Box<dyn Scale<f64, f64>>`)
- Different scale types can't share common storage

## Approach 2: Enum-Based Output (Semi-Dynamic)
```rust
pub enum ScaleOutput {
    Continuous(f64),      // For position, size, alpha
    Color(Color),          // For color aesthetics
    Shape(PointShape),     // For shape aesthetics
}

pub trait Scale: Send + Sync {
    fn domain_type(&self) -> DomainType; // Continuous or Discrete
    fn range_type(&self) -> RangeType;   // Continuous, Color, Shape, etc.
    
    // Continuous domain
    fn map_continuous(&self, value: f64) -> Option<ScaleOutput>;
    
    // Discrete domain
    fn map_discrete(&self, value: &str) -> Option<ScaleOutput>;
    
    fn train(&mut self, data: &dyn GenericVector);
}
```

**Pros:**
- Single trait, easier to store in ScaleSet
- Runtime type checking
- Flexible for different aesthetic types

**Cons:**
- Runtime overhead for enum matching
- Less type safety
- Can call wrong method if not careful

## Approach 3: Separate Scale Traits by Output (Recommended)
```rust
// Base trait for all scales
pub trait ScaleBase: Send + Sync {
    fn train(&mut self, data: &dyn GenericVector);
    fn breaks(&self) -> Vec<String>; // Break labels
}

// Scales that map to continuous [0,1] (position, size, alpha)
pub trait ContinuousScale: ScaleBase {
    fn map_continuous(&self, value: f64) -> Option<f64>;
    fn inverse(&self, value: f64) -> f64;
}

pub trait DiscreteScale: ScaleBase {
    fn map_discrete(&self, value: &str) -> usize; // Maps to index
}

// Scales that map to colors
pub trait ColorScale: ScaleBase {
    fn map_to_color(&self, value: &dyn Any) -> Option<Color>;
}

// Scales that map to shapes
pub trait ShapeScale: ScaleBase {
    fn map_to_shape(&self, value: &dyn Any) -> Option<PointShape>;
}
```

**Pros:**
- Clear separation of concerns
- Type safe for each aesthetic type
- Can store different scale types appropriately
- Follows ggplot2's conceptual model

**Cons:**
- More traits to implement
- ScaleSet needs different fields for different scale types
- Some duplication between continuous/discrete variants

## Approach 4: Specialized Trait per Aesthetic (Most Flexible)
```rust
pub trait PositionScale: Send + Sync {
    fn map_to_normalized(&self, data: &dyn GenericVector) -> Vec<f64>;
    fn train(&mut self, data: &dyn GenericVector);
}

pub trait ColorScale: Send + Sync {
    fn map_to_colors(&self, data: &dyn GenericVector) -> Vec<Color>;
    fn train(&mut self, data: &dyn GenericVector);
}

pub trait ShapeScale: Send + Sync {
    fn map_to_shapes(&self, data: &dyn GenericVector) -> Vec<PointShape>;
    fn train(&mut self, data: &dyn GenericVector);
}
```

**Pros:**
- Each aesthetic gets exactly what it needs
- Scales handle vectorization internally
- Very flexible - continuous→discrete for colors works naturally
- Matches how RenderContext actually uses scales

**Cons:**
- More traits
- Less code reuse between scale types
- Larger API surface

## Recommendation: Hybrid Approach

Use Approach 3 as the base, but with improvements:

1. **Position/size/alpha scales** remain as now (map f64 → Option<f64>)
2. **Color scales** have their own trait that can handle continuous or discrete input
3. **Shape scales** typically discrete only
4. **RenderContext methods** dispatch to the appropriate scale type

This matches ggplot2's model where `scale_color_continuous()`, `scale_color_discrete()`, 
`scale_color_gradient()` etc. are distinct scale types that all produce colors but from 
different domains.

The key insight: **the aesthetic determines the output type**, while **the data and 
scale choice determine the mapping**. So we should organize scales by what they produce, 
not just by mathematical properties.
