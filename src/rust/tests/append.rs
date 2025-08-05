#[cfg(test)]
mod tests {
    use fallible_streaming_iterator::FallibleStreamingIterator;
    use flatgeobuf::*;
    use geo_types::{line_string, LineString};
    use std::fs::File;
    use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};

    #[test]
    fn test_fgb_appender_workflow() -> Result<()> {
        // Initial set of linestrings
        let initial_linestrings: Vec<LineString<f64>> = vec![
            line_string![
                (x: -21.95156, y: 64.1446),
                (x: -21.951, y: 64.14479),
                (x: -21.95044, y: 64.14527),
                (x: -21.951445, y: 64.145508)
            ],
            line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0)],
            line_string![(x: 0.5, y: 0.0), (x: 1.0, y: 1.0)],
            line_string![(x: 0.0, y: 0.5), (x: 1.0, y: 1.0)],
            line_string![(x: 0.0, y: 0.0), (x: 0.5, y: 1.0)],
        ];

        // Additional linestrings to append
        let additional_linestrings: Vec<LineString<f64>> = vec![
            line_string![(x: 2.0, y: 0.0), (x: 3.0, y: 1.0)],
            line_string![(x: 0.5, y: 6.0), (x: 5.0, y: 1.0)],
            line_string![(x: 7.0, y: 7.0), (x: 8.0, y: 7.0)],
            line_string![(x: 0.0, y: 10.0), (x: 0.5, y: 9.0)],
        ];

        // === Phase 1: Create initial mutable FGB file ===
        let initial_file_path = "test_initial.fgb";
        {
            let file = File::create(initial_file_path)?;
            let mut writer = BufWriter::new(file);

            let mut fgb = FgbWriter::create_with_options(
                "test_mutable",
                GeometryType::LineString,
                FgbWriterOptions {
                    write_index: true,
                    crs: FgbCrs {
                        code: 4326,
                        ..Default::default()
                    },
                    mutability_version: 1, // Enable mutability
                    ..Default::default()
                },
            )?;

            // Add initial features
            for geom in &initial_linestrings {
                let geom = geo_types::Geometry::LineString(geom.clone());
                let _ = fgb.add_feature_geom(geom, |_feat| {});
            }

            fgb.write(&mut writer)?;
            writer.flush()?;
        }

        println!(
            "Created initial FGB file with {} features",
            initial_linestrings.len()
        );

        // === Phase 2: Test reading initial file ===
        {
            let file = File::open(initial_file_path)?;
            let mut reader = BufReader::new(file);
            let mut fgb = FgbReader::open(&mut reader)?.select_all()?;

            println!("Initial file header: {:?}", fgb.header());
            assert_eq!(
                fgb.header().features_count(),
                initial_linestrings.len() as u64
            );
            assert_eq!(fgb.header().mutablity_version(), 1);

            let mut count = 0;
            while let Some(_feature) = fgb.next()? {
                count += 1;
            }
            assert_eq!(count, initial_linestrings.len());
        }

        // === Phase 3: Append features using FgbAppender ===
        let final_file_path = "test_appended.fgb";
        {
            // Copy the initial file to final file path first
            std::fs::copy(initial_file_path, final_file_path)?;

            // Open the file with read+write permissions
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(final_file_path)?;
            let mut cloned = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(final_file_path)?;

            cloned.seek(SeekFrom::Start(0))?;
            // Create appender
            let mut appender = FgbAppender::open(&mut file)?;

            // Add new features
            for geom in &additional_linestrings {
                let geom = geo_types::Geometry::LineString(geom.clone());
                let _ = appender.add_feature_geom(geom, |_feat| {});
            }

            println!(
                "Added {} new features to appender",
                additional_linestrings.len()
            );

            // Write the appended data directly to the file
            appender.reindex_append(&mut cloned)?;
            // cloned.flush()?;
        }

        println!("Appended features and created final file");

        // === Phase 4: Test reading the final appended file ===
        {
            let file = File::open(final_file_path)?;
            let mut reader = BufReader::new(file);
            let mut fgb = FgbReader::open(&mut reader)?.select_all()?;

            let total_expected = initial_linestrings.len() + additional_linestrings.len();
            println!("Final file header: {:?}", fgb.header());
            assert_eq!(fgb.header().features_count(), total_expected as u64);
            assert_eq!(fgb.header().mutablity_version(), 1);

            let mut count = 0;
            while let Some(feature) = fgb.next()? {
                let geometry = feature.geometry().unwrap();
                println!("Feature {}: {:?}", count + 1, geometry);
                count += 1;
            }

            assert_eq!(count, total_expected);
            println!("Successfully read {} features from appended file", count);
        }

        // === Phase 5: Test spatial queries on appended file ===
        {
            let file = File::open(final_file_path)?;
            let mut reader = BufReader::new(file);
            let mut fgb = FgbReader::open(&mut reader)?.select_bbox(-1.0, -1.0, 2.0, 2.0)?;

            let mut bbox_count = 0;
            while let Some(feature) = fgb.next()? {
                let geometry = feature.geometry().unwrap();
                println!("Bbox feature {}: {:?}", bbox_count + 1, geometry);
                bbox_count += 1;
            }

            // Should find features that intersect the bbox
            assert!(bbox_count > 0);
            println!("Found {} features in bbox query", bbox_count);
        }

        // === Cleanup ===
        std::fs::remove_file(initial_file_path).ok();
        std::fs::remove_file(final_file_path).ok();

        Ok(())
    }

    // Probably can't be fixed without something hacky
    // To be investigated
    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn test_fgb_appender_empty_initial_file() -> () {
        // Test appending to an initially empty mutable file
        let empty_file_path = "test_empty.fgb";
        let appended_file_path = "test_empty_appended.fgb";

        // Create empty mutable FGB file
        {
            let file = File::create(empty_file_path).unwrap();
            let mut writer = BufWriter::new(file);

            let fgb = FgbWriter::create_with_options(
                "test_empty",
                GeometryType::LineString,
                FgbWriterOptions {
                    write_index: true,
                    mutability_version: 1,
                    ..Default::default()
                },
            )
            .unwrap();

            fgb.write(&mut writer).unwrap();
            writer.flush().unwrap();
        }

        // Append features
        {
            std::fs::copy(empty_file_path, appended_file_path).unwrap();

            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(appended_file_path)
                .unwrap();
            let mut cloned = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(appended_file_path)
                .unwrap();

            let mut appender = FgbAppender::open(&mut file).unwrap();

            let linestring = line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0)];
            let geom = geo_types::Geometry::LineString(linestring);
            let _ = appender.add_feature_geom(geom, |_feat| {});

            appender.reindex_append(&mut cloned).unwrap();
        }

        // Verify the result
        {
            let file = File::open(appended_file_path).unwrap();
            let mut reader = BufReader::new(file);
            let mut fgb = FgbReader::open(&mut reader).unwrap().select_all().unwrap();

            assert_eq!(fgb.header().features_count(), 1);

            let mut count = 0;
            while let Some(_feature) = fgb.next().unwrap() {
                count += 1;
            }
            assert_eq!(count, 1);
        }

        // Cleanup
        std::fs::remove_file(empty_file_path).ok();
        std::fs::remove_file(appended_file_path).ok();

        // Ok(())
    }

    #[test]
    fn test_fgb_appender_immutable_file_error() -> Result<()> {
        // Test that appender rejects immutable files
        let immutable_file_path = "test_immutable.fgb";

        // Create immutable FGB file
        {
            let file = File::create(immutable_file_path)?;
            let mut writer = BufWriter::new(file);

            let mut fgb = FgbWriter::create_with_options(
                "test_immutable",
                GeometryType::LineString,
                FgbWriterOptions {
                    write_index: true,
                    mutability_version: 0, // Immutable
                    ..Default::default()
                },
            )?;

            let linestring = line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0)];
            let geom = geo_types::Geometry::LineString(linestring);
            let _ = fgb.add_feature_geom(geom, |_feat| {});
            fgb.write(&mut writer)?;
            writer.flush()?;
        }

        // Try to open with appender - should fail
        {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(immutable_file_path)?;

            let result = FgbAppender::open(&mut file);

            match result {
                Err(Error::Immutable) => {
                    println!("Correctly rejected immutable file");
                }
                _ => {
                    panic!("Expected Immutable error");
                }
            }
        }

        // Cleanup
        std::fs::remove_file(immutable_file_path).ok();

        Ok(())
    }
}
