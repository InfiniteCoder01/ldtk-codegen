use crate::definitions::*;

pub fn generate_defs(
    preferences: &Preferences,
    definitions: &mut RsDefinitions,
    project: &LdtkJson,
    code: &mut Scope,
) -> Result<()> {
    let header = include_str!("header_template.rs")
        .replace(
            "[SERDE]",
            if preferences.serde() {
                "Serialize, Deserialize, "
            } else {
                ""
            },
        )
        .trim()
        .to_owned();
    let header = if preferences.vec2s().is_empty() {
        header.replace("\n    CUSTOM_VECTORS!();", "")
    } else {
        header.replace(
            "CUSTOM_VECTORS!();",
            &format!("generate_from_into!({});", preferences.vec2s().join(", ")),
        )
    };
    let header = if preferences.colors().is_empty() {
        header.replace("\n    CUSTOM_COLORS!();", "")
    } else {
        header.replace(
            "CUSTOM_COLORS!();",
            &format!(
                "generate_color_from_into!({});",
                preferences.colors().join(", ")
            ),
        )
    };
    macro_rules! replace_get_layer {
        ($header: ident, $mutability: literal) => {
            $header.replace(
                concat!(
                    "            LAYER_INDEX => GET_LAYER",
                    $mutability,
                    "!(),\n"
                ),
                &project
                    .defs
                    .layers
                    .iter()
                    .filter(|layer| matches!(layer.purple_type, Type::Entities))
                    .enumerate()
                    .map(|(index, layer)| {
                        format!(
                            concat!(
                                "            {} => level.{}.get",
                                $mutability,
                                "(self.entity),\n"
                            ),
                            index,
                            preferences.to_case(&layer.identifier, Case::Snake)
                        )
                    })
                    .collect::<String>(),
            )
        };
    }
    let header = replace_get_layer!(header, "");
    let header = replace_get_layer!(header, "_mut");
    code.raw(header);

    for tileset in &project.defs.tilesets {
        definitions.tilesets.insert(
            tileset.uid,
            RsTilesetDefinition {
                tile_size: tileset.tile_grid_size as _,
            },
        );
    }

    code.raw("/* --- Definitions --- */");
    code.raw("/* Enums */");
    for enum_json in &project.defs.enums {
        let enum_rs = code.new_enum(&enum_json.identifier).vis("pub");
        derive_rust_object!(enum_rs preferences.serde(), Hash !partial Eq, Ord);
        for value in &enum_json.values {
            enum_rs.new_variant(&value.id);
        }

        macro_rules! generate_get_const {
            ($impl: ident $fn: ident -> $ret: ty; $variant: ident => $line: expr) => {
                $impl
                    .new_fn(stringify!($fn))
                    .vis("pub")
                    .arg_ref_self()
                    .ret(stringify!($ret))
                    .push_block({
                        let mut match_block = Block::new("match self");
                        for $variant in &enum_json.values {
                            match_block.line(format!("Self::{} => {},", $variant.id, $line));
                        }
                        match_block
                    });
            };
        }

        if let Some(tileset) = enum_json.icon_tileset_uid {
            let enum_impl = code.new_impl(&enum_json.identifier);
            enum_impl.associate_const("TILESET_ID", "TilesetID", tileset.to_string(), "pub");
            generate_get_const!(enum_impl icon -> Option<Vec2<u32>>; variant => if let Some(tile) = &variant.tile_rect {
                let tileset = definitions.tilesets.get(&tileset).context("Enum icon tileset was not found!")?;
                format!("Some(Vec2::new({}, {}))", tile.x as u32 / tileset.tile_size, tile.y as u32 / tileset.tile_size)
            } else {
                "None".to_owned()
            });
        }
    }
    Ok(())
}
