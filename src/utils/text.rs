use std::ops::Deref;
use std::fmt;
use internment::ArcIntern;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Text(ArcIntern<String>);

impl Deref for Text {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Text(ArcIntern::new(s))
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Text(ArcIntern::new(s.to_string()))
    }
}

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialOrd for Text {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}
impl Ord for Text {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_pointer() {
        let t1 = Text::from("same");
        let t2 = Text::from("same");
        let t3 = Text::from("different");
        assert_eq!(t1, t2); // interned, should be pointer-equal
        assert_ne!(t1, t3);
        // Pointer equality: internment guarantees same pointer for same string
        assert!(std::ptr::eq(t1.0.as_ref(), t2.0.as_ref()));
    }

    #[test]
    fn test_ord() {
        let t1 = Text::from("apple");
        let t2 = Text::from("banana");
        let t3 = Text::from("apple");
        assert!(t1 < t2);
        assert!(t2 > t1);
        assert_eq!(t1, t3);
        let mut v = vec![t2.clone(), t1.clone()];
        v.sort();
        assert_eq!(v, vec![t1, t2]);
    }

    #[test]
    fn test_deref() {
        let t = Text::from("hello world");
        assert_eq!(&*t, "hello world");
    }

    #[test]
    fn test_as_ref() {
        let t = Text::from("foo bar");
        assert_eq!(t.as_ref(), "foo bar");
    }

    #[test]
    fn test_display() {
        let t = Text::from("display me");
        assert_eq!(format!("{}", t), "display me");
    }
}