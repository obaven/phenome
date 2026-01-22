use std::time::Duration;

use crate::scaling::scaling_prediction::ScalingPredictor;

#[test]
fn predicts_average_for_history() {
    let predictor = ScalingPredictor::new();
    let prediction = predictor
        .predict(
            "deployment-a".to_string(),
            Duration::from_secs(3600),
            &[1.0, 2.0, 3.0],
            "cores",
            0,
        )
        .unwrap();

    assert_eq!(prediction.predicted_value, 2.0);
}
