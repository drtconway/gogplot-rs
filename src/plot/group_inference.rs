// Group inference for position adjustments

use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::scale::ScaleType;

/// Infer the Group aesthetic from fill or color if not explicitly set.
/// This makes position adjustments simpler - they can always look for Group.
/// 
/// If Group is already mapped, does nothing.
/// If fill or color is mapped to a categorical column, use that for Group.
pub fn infer_group_aesthetic(mapping: &mut AesMap) {
    // If Group is already explicitly set, nothing to do
    if mapping.contains(Aesthetic::Group) {
        return;
    }
    
    // Check if fill is categorical
    if let Some(AesValue::Column { name, hint: Some(ScaleType::Categorical) }) = 
        mapping.get(&Aesthetic::Fill) 
    {
        mapping.set(Aesthetic::Group, AesValue::Column {
            name: name.clone(),
            hint: Some(ScaleType::Categorical),
        });
        return;
    }
    
    // Check if color is categorical
    if let Some(AesValue::Column { name, hint: Some(ScaleType::Categorical) }) = 
        mapping.get(&Aesthetic::Color) 
    {
        mapping.set(Aesthetic::Group, AesValue::Column {
            name: name.clone(),
            hint: Some(ScaleType::Categorical),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_infer_from_fill() {
        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::Fill, AesValue::Column {
            name: "category".to_string(),
            hint: Some(ScaleType::Categorical),
        });
        
        infer_group_aesthetic(&mut mapping);
        
        assert!(mapping.contains(Aesthetic::Group));
        if let Some(AesValue::Column { name, .. }) = mapping.get(&Aesthetic::Group) {
            assert_eq!(name, "category");
        } else {
            panic!("Expected Group to be set to column");
        }
    }
    
    #[test]
    fn test_infer_from_color() {
        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::Color, AesValue::Column {
            name: "group".to_string(),
            hint: Some(ScaleType::Categorical),
        });
        
        infer_group_aesthetic(&mut mapping);
        
        assert!(mapping.contains(Aesthetic::Group));
    }
    
    #[test]
    fn test_no_inference_when_explicit() {
        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::Group, AesValue::Column {
            name: "explicit_group".to_string(),
            hint: None,
        });
        mapping.set(Aesthetic::Fill, AesValue::Column {
            name: "category".to_string(),
            hint: Some(ScaleType::Categorical),
        });
        
        infer_group_aesthetic(&mut mapping);
        
        // Should keep the explicit group
        if let Some(AesValue::Column { name, .. }) = mapping.get(&Aesthetic::Group) {
            assert_eq!(name, "explicit_group");
        } else {
            panic!("Expected Group to remain as explicit_group");
        }
    }
    
    #[test]
    fn test_no_inference_for_continuous() {
        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::Fill, AesValue::Column {
            name: "value".to_string(),
            hint: Some(ScaleType::Continuous),
        });
        
        infer_group_aesthetic(&mut mapping);
        
        // Should not infer Group from continuous aesthetic
        assert!(!mapping.contains(Aesthetic::Group));
    }
}
