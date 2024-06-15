//spawn is 986,84,778
// 9 cubes of fake2.dev are surrounding the spawn point

pub fn get_region_json() -> serde_json::Value {
    serde_json::json!({
        "regions": [
            {
                "cubes": [
                    {
                        "center": [950, 100, 750],
                        "radius": 50
                    },
                ],
                "owner": "fake.dev"
            },
            {
                "cubes": [
                    {
                        "center": [850, 100, 650],
                        "radius": 50
                    },
                    {
                        "center": [850, 100, 750],
                        "radius": 50
                    },
                    {
                        "center": [850, 100, 850],
                        "radius": 50
                    },
                    {
                        "center": [950, 100, 650],
                        "radius": 50
                    },
                    {
                        "center": [950, 100, 750],
                        "radius": 50
                    },
                    {
                        "center": [950, 100, 850],
                        "radius": 50
                    },
                    {
                        "center": [1050, 100, 650],
                        "radius": 50
                    },
                    {
                        "center": [1050, 100, 750],
                        "radius": 50
                    },
                    {
                        "center": [1050, 100, 850],
                        "radius": 50
                    }
                ],
                "owner": "fake2.dev"
            }
        ]
    })
}
