# Grouped Histograms and Bar Charts - Implementation Plan

## Current Status

Running `examples/grouped_histogram.rs` fails with:
```
Error: InvalidAestheticType("Fill mapped from column requires a color scale")
```

This reveals that grouped/colored histograms and bar charts need several features:

## What's Needed

### 1. Fill Scales
Currently only Color scales exist. Need to add:
- `scale_fill_discrete()` / `scale_fill_categorical()` 
- `scale_fill_continuous()`
- `scale_fill_manual()`

In ggplot2, `color` and `fill` are separate aesthetics:
- `color` - outline/stroke color
- `fill` - fill color

### 2. Grouped Binning in Stat::Bin
When `fill` or `color` is mapped to a grouping variable, the bin stat needs to:
- Detect the grouping aesthetic
- Bin each group separately
- Generate separate x/count/xmin/xmax rows for each group
- Preserve the group column in the output

### 3. Position Adjustments
Three key position adjustments are needed:

#### Position::Identity (already exists)
- Draw bars at their actual positions
- For grouped histograms: bars overlap (need alpha transparency)

#### Position::Stack
- Stack bars on top of each other
- For histograms: cumulative counts
- For bar charts: total = sum of groups

#### Position::Dodge
- Place bars side-by-side
- Each bar is narrower to fit multiple groups
- For histograms: bars are dodged within each bin

### 4. Rendering Updates

#### GeomHistogram::render()
Currently assumes single series. Needs to:
- Handle multiple rows per bin (one per group)
- Apply position adjustments
- Use fill scale to map group -> color

#### GeomBar::render()
Similar updates needed for bar charts

### 5. Scale Set
The `ScaleSet` in `src/plot/scale_set.rs` needs:
- `fill` scale field (currently only has x, y, color, shape, size)
- Training logic for fill aesthetic
- Default fill scale creation

## Implementation Priority

Minimal viable feature:
1. Add Fill scale support (reuse Color scale logic)
2. Update ScaleSet to handle fill
3. Modify Stat::Bin to preserve group columns
4. Simple identity positioning with overlapping bars

Full feature:
5. Implement Position::Stack
6. Implement Position::Dodge  
7. Handle grouped binning in Stat::Bin
8. Add automatic legends for grouped aesthetics

## Design Questions

1. **Should fill and color use the same palette by default?**
   - In ggplot2, they're independent but often use same colors
   - Could share categorical palette but allow override

2. **How to specify grouping?**
   - Implicit: any aesthetic mapped to categorical data creates groups
   - Explicit: dedicated `group` aesthetic
   - Both? (ggplot2 uses both)

3. **Default position for histogram vs bar?**
   - Histogram: identity (overlapping) or stack?
   - Bar: stack (most common) or identity?

4. **Bin alignment for grouped histograms?**
   - Should all groups use same bin boundaries? (Yes - essential for comparison)
   - Bin based on combined data range, then count per group

## Example Use Cases

```rust
// Overlapping histograms with transparency
Plot::new(data)
    .aes(|a| { a.x("value"); a.fill("group"); })
    .geom_histogram_with(|g| g.bins(20).alpha(0.5))  // position = identity default
    
// Stacked histogram
Plot::new(data)
    .aes(|a| { a.x("value"); a.fill("group"); })
    .geom_histogram_with(|g| g.bins(20).position(Position::Stack))
    
// Side-by-side histogram
Plot::new(data)
    .aes(|a| { a.x("value"); a.fill("group"); })
    .geom_histogram_with(|g| g.bins(20).position(Position::Dodge))
    
// Grouped bar chart
Plot::new(data)
    .aes(|a| { a.x("category"); a.fill("group"); })
    .geom_bar()  // Stat::Count, Position::Stack by default?
```

## Next Steps

Start with simplest path:
1. Implement fill scales (copy/adapt color scale code)
2. Get overlapping histograms working (identity position, alpha blending)
3. Then tackle position adjustments (stack, dodge)
