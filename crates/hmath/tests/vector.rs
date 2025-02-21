#[cfg(test)]
mod tests {
    use hmath::vector::Vector3;

    #[test]
    fn test_vector3_new() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_vector3_dot() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        assert_eq!(v1.dot(&v2), 32.0);
    }

    #[test]
    fn test_vector3_cross() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        let cross = v1.cross(&v2);
        assert_eq!(cross.x, -3.0);
        assert_eq!(cross.y, 6.0);
        assert_eq!(cross.z, -3.0);
    }

    #[test]
    fn test_vector3_length() {
        let v = Vector3::new(1.0, 2.0, 2.0);
        assert_eq!(v.length(), 3.0);
    }

    /*#[test]
    fn test_vector3_normalize() {
        let v = Vector3::new(1.0, 2.0, 2.0);
        let normalized = v.normalize();
        assert!((normalized.x - 1.0 / 3.0).abs() < 1e-6);
        assert!((normalized.y - 2.0 / 3.0).abs() < 1e-6);
        assert!((normalized.z - 2.0 / 3.0).abs() < 1e-6);
    }*/

    #[test]
    fn test_vector3_add() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        let sum = v1 + v2;
        assert_eq!(sum.x, 5.0);
        assert_eq!(sum.y, 7.0);
        assert_eq!(sum.z, 9.0);
    }

    #[test]
    fn test_vector3_sub() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        let diff = v1 - v2;
        assert_eq!(diff.x, -3.0);
        assert_eq!(diff.y, -3.0);
        assert_eq!(diff.z, -3.0);
    }

    #[test]
    fn test_vector3_mul() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let scaled = v * 2.0;
        assert_eq!(scaled.x, 2.0);
        assert_eq!(scaled.y, 4.0);
        assert_eq!(scaled.z, 6.0);
    }

    #[test]
    fn test_vector3_div() {
        let v = Vector3::new(2.0, 4.0, 6.0);
        let scaled = v / 2.0;
        assert_eq!(scaled.x, 1.0);
        assert_eq!(scaled.y, 2.0);
        assert_eq!(scaled.z, 3.0);
    }
}