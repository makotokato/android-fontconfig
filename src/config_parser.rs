// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate xml;

use std::fs::File;
use std::io::BufReader;
use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};

#[derive(Clone)]
struct FontAlias {
    name: String,
    to: String,
    weight: Option<i32>,
}

impl FontAlias {
    pub fn new() -> FontAlias {
        FontAlias {
            name: "".to_string(),
            to: "".to_string(),
            weight: None,
        }
    }
}

#[derive(Clone)]
struct FontAxis {
    tag: String,
    stylevalue: f64,
}

impl FontAxis {
    pub fn new() -> FontAxis {
        FontAxis {
            tag: "".to_string(),
            stylevalue: 0.0,
        }
    }
}

#[derive(Clone)]
struct FontEntry {
    path: Option<String>,
    weight: Option<i32>,
    italic: bool,
    fallback_for: Option<String>,
    index: i32,
    axis: Vec<FontAxis>,
}

impl FontEntry {
    pub fn new() -> FontEntry {
        FontEntry {
            path: None,
            weight: None,
            italic: false,
            fallback_for: None,
            index: 0,
            axis: Vec::new(),
        }
    }

    pub fn is_regular(&self) -> bool {
        if self.italic {
            return false;
        }
        if let Some(w) = self.weight {
            return w == 400;
        }
        true
    }
}

#[derive(Clone)]
struct FontFamily {
    name: Option<String>,
    lang: Option<String>,
    variant: Option<String>,
    fonts: Vec<FontEntry>,
}

impl FontFamily {
    pub fn new() -> FontFamily {
        let fonts = Vec::new();
        FontFamily {
            name: None,
            lang: None,
            variant: None,
            fonts: fonts,
        }
    }
}

impl PartialEq for FontFamily {
    fn eq(&self, other: &Self) -> bool {
        (self.lang == other.lang && self.name.is_none() && other.name.is_none())
            || (self.name.is_some() && other.name.is_some() && self.name == other.name)
    }
}

pub struct AndroidFontConfig {
    font_families: Vec<FontFamily>,
    font_aliases: Vec<FontAlias>,
}

#[allow(dead_code)]
impl AndroidFontConfig {
    pub fn new() -> AndroidFontConfig {
        let (families, aliases) = AndroidFontConfig::parse("/etc/fonts.xml");
        AndroidFontConfig {
            font_families: families,
            font_aliases: aliases,
        }
    }

    #[cfg(test)]
    pub fn new_from_file(config_xml: &str) -> AndroidFontConfig {
        let (families, aliases) = AndroidFontConfig::parse(config_xml);
        AndroidFontConfig {
            font_families: families,
            font_aliases: aliases,
        }
    }

    fn parse_alias(attributes: &Vec<OwnedAttribute>) -> FontAlias {
        let mut font_alias = FontAlias::new();

        for attr in attributes {
            match attr.name.local_name.as_str() {
                "name" => {
                    font_alias.name = attr.value.clone();
                }
                "to" => {
                    font_alias.to = attr.value.clone();
                }
                "weight" => {
                    font_alias.weight = Some(attr.value.parse().unwrap());
                }
                _ => {}
            }
        }
        font_alias
    }

    fn parse_family(attributes: &Vec<OwnedAttribute>) -> FontFamily {
        let mut font_family = FontFamily::new();

        for attr in attributes {
            match attr.name.local_name.as_str() {
                "lang" => {
                    font_family.lang = Some(attr.value.clone());
                }
                "name" => {
                    font_family.name = Some(attr.value.clone());
                }
                "variant" => {
                    font_family.variant = Some(attr.value.clone());
                }
                _ => {}
            }
        }
        font_family
    }

    fn parse_font(attributes: &Vec<OwnedAttribute>) -> FontEntry {
        let mut font = FontEntry::new();

        for attr in attributes {
            match attr.name.local_name.as_str() {
                "weight" => {
                    font.weight = Some(attr.value.parse().unwrap());
                }

                "style" => match attr.value.as_str() {
                    "normal" => {
                        font.italic = false;
                    }
                    "italic" => {
                        font.italic = true;
                    }
                    _ => {}
                },
                "fallbackFor" => {
                    font.fallback_for = Some(attr.value.clone());
                }
                "index" => {
                    font.index = attr.value.parse().unwrap();
                }
                _ => {}
            }
        }
        font
    }

    fn parse_axis(attributes: &Vec<OwnedAttribute>) -> FontAxis {
        let mut axis = FontAxis::new();

        for attr in attributes {
            match attr.name.local_name.as_str() {
                "tag" => {
                    axis.tag = attr.value.clone();
                }

                "stylevalue" => {
                    axis.stylevalue = attr.value.parse().unwrap();
                }
                _ => {}
            }
        }
        axis
    }

    fn parse(config_xml_path: &str) -> (Vec<FontFamily>, Vec<FontAlias>) {
        let file = BufReader::new(File::open(config_xml_path).unwrap());
        let parser = EventReader::new(file);

        let mut current_elements = Vec::new();
        let mut font_families = Vec::new();
        let mut font_aliases = Vec::new();

        let mut font = FontEntry::new();
        let mut family = FontFamily::new();

        for e in parser {
            match e {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    current_elements.push(name.local_name.clone());
                    match name.local_name.as_str() {
                        "alias" => {
                            font_aliases.push(AndroidFontConfig::parse_alias(&attributes));
                        }

                        "family" => {
                            family = AndroidFontConfig::parse_family(&attributes);
                        }

                        "font" => {
                            font = AndroidFontConfig::parse_font(&attributes);
                        }

                        "axis" => {
                            font.axis.push(AndroidFontConfig::parse_axis(&attributes));
                        }

                        _ => {}
                    }
                }
                Ok(XmlEvent::EndElement { name }) => {
                    current_elements.pop();

                    match name.local_name.as_str() {
                        "font" => {
                            if font.path.is_some() && current_elements.last().unwrap() == "family" {
                                family.fonts.push(font.clone());
                            }
                        }
                        "family" => {
                            if let Some(family_lang) = &family.lang {
                                let lang_attr = family_lang.clone();
                                if lang_attr.contains(",") {
                                    let v: Vec<&str> = lang_attr.split(",").collect();
                                    for lang in v {
                                        let mut f = family.clone();
                                        f.lang = Some(lang.to_string());
                                        font_families.push(f);
                                    }
                                } else if lang_attr.contains(" ") {
                                    let v: Vec<&str> = lang_attr.split(" ").collect();
                                    for lang in v {
                                        let mut f = family.clone();
                                        f.lang = Some(lang.to_string());
                                        font_families.push(f);
                                    }
                                } else {
                                    font_families.push(family.clone());
                                }
                            } else {
                                font_families.push(family.clone());
                            }
                        }
                        _ => {}
                    }
                }
                Ok(XmlEvent::Characters(s)) => {
                    if current_elements.last().unwrap() == "font" {
                        font.path = Some("/system/fonts/".to_owned() + &s.trim());
                    }
                }
                _ => {}
            }
        }
        (font_families, font_aliases)
    }

    /// Return font family name of default font.
    pub fn default_family_name(&self) -> String {
        for family in &self.font_families {
            if let Some(v) = &family.name {
                return v.clone();
            }
        }
        "".to_owned()
    }

    /// Return font family by resolving alias name
    fn resolve_font_family_by_alias<'a>(&'a self, name: &'a str) -> &'a str {
        for alias in &self.font_aliases {
            if alias.name == name {
                return &alias.to;
            }
        }
        name
    }

    /// Return font path by font family and language
    pub fn font_path_by_family_and_lang(
        &self,
        name: &str,
        lang: &str,
    ) -> Result<(&str, i32), String> {
        for family in &self.font_families {
            if family.lang.is_some() && family.lang.as_ref().unwrap() == lang {
                for font in &family.fonts {
                    if let Some(fallback) = &font.fallback_for {
                        if fallback == name {
                            return Ok((font.path.as_ref().unwrap(), font.index));
                        }
                    } else if name == "sans-serif" {
                        return Ok((font.path.as_ref().unwrap(), font.index));
                    }
                }
            }
        }
        Err("not found".to_string())
    }

    /// Return font path of default font by language
    pub fn default_font_path_by_lang(&self, lang: &str) -> Result<(&str, i32), &'static str> {
        for family in &self.font_families {
            if let Some(font_lang) = &family.lang {
                if font_lang == lang {
                    for font in &family.fonts {
                        if font.is_regular() && font.path.is_some() {
                            return Ok((font.path.as_ref().unwrap(), font.index));
                        }
                    }
                }
            } else if lang.is_empty() {
                for font in &family.fonts {
                    if font.is_regular() && font.path.is_some() {
                        return Ok((font.path.as_ref().unwrap(), font.index));
                    }
                }
            }
        }
        Err("not found")
    }

    /// Return all font paths.
    pub fn all_font_paths(&self) -> Vec<(String, i32)> {
        self.font_families
            .iter()
            .map(|f| &f.fonts)
            .flatten()
            .collect::<Vec<_>>()
            .iter()
            .filter(|font| font.path.is_some())
            .map(|font| (font.path.clone().unwrap(), font.index))
            .collect()
    }

    /// Return all font families
    pub fn all_families(&self) -> Vec<String> {
        let mut families: Vec<String> = self
            .font_families
            .iter()
            .filter(|f| f.name.is_some())
            .map(|f| f.name.clone().unwrap())
            .collect();
        families.append(&mut self.font_aliases.iter().map(|f| f.name.clone()).collect());
        families
    }

    /// Return all fonts by family name
    pub fn select_family_by_name(
        &self,
        family_name: &str,
    ) -> Result<Vec<(String, i32)>, &'static str> {
        let family_name = self.resolve_font_family_by_alias(family_name);
        let mut paths: Vec<(String, i32)> = vec![];
        for family in &self.font_families {
            if let Some(name) = &family.name {
                if family_name == name {
                    family
                        .fonts
                        .iter()
                        .filter(|font| font.path.is_some() && font.is_regular())
                        .for_each(|font| paths.push((font.path.clone().unwrap(), font.index)));
                }
            } else {
                family
                    .fonts
                    .iter()
                    .filter(|font| {
                        (font.fallback_for.is_some()
                            && font.fallback_for.as_ref().unwrap() == family_name)
                            || (font.fallback_for.is_none() && family_name == "sans-serif")
                    })
                    .for_each(|font| paths.push((font.path.clone().unwrap(), font.index)));
            }
        }
        if paths.len() > 0 {
            return Ok(paths);
        }
        Err("not found")
    }
}

#[cfg(test)]
#[test]
fn test_default_font() {
    let config = AndroidFontConfig::new_from_file("data/fonts-1.xml");
    assert_eq!(config.default_family_name(), "sans-serif");
    assert_eq!(
        config.default_font_path_by_lang("").unwrap(),
        ("/system/fonts/Roboto-Regular.ttf", 0)
    );
}

#[cfg(test)]
#[test]
fn test_select_family_by_name() {
    let config = AndroidFontConfig::new_from_file("data/fonts-1.xml");
    assert!(config
        .select_family_by_name("sans-serif")
        .unwrap()
        .contains(&("/system/fonts/Roboto-Regular.ttf".to_owned(), 0)));

    assert!(config
        .select_family_by_name("sans-serif")
        .unwrap()
        .contains(&("/system/fonts/NotoSansCJK-Regular.ttc".to_owned(), 0)));
    assert!(config
        .select_family_by_name("sans-serif")
        .unwrap()
        .contains(&("/system/fonts/NotoSansCJK-Regular.ttc".to_owned(), 2)));

    assert!(config
        .select_family_by_name("serif")
        .unwrap()
        .contains(&("/system/fonts/NotoSerif-Regular.ttf".to_owned(), 0)));
    assert!(config
        .select_family_by_name("serif")
        .unwrap()
        .contains(&("/system/fonts/NotoSerifThai-Regular.ttf".to_owned(), 0)));

    // Shouldn't match
    assert!(!config
        .select_family_by_name("serif")
        .unwrap()
        .contains(&("/system/fonts/NotoSansCJK-Regular.ttc".to_owned(), 0)));
    assert!(!config
        .select_family_by_name("sans-serif")
        .unwrap()
        .contains(&("/system/fonts/Roboto-Thin.ttf".to_owned(), 0)));

    // Alias
    assert!(config
        .select_family_by_name("arial")
        .unwrap()
        .contains(&("/system/fonts/Roboto-Regular.ttf".to_owned(), 0)));
}

#[cfg(test)]
#[test]
fn test_fallback_lang() {
    let config = AndroidFontConfig::new_from_file("data/fonts-1.xml");
    // Fallback entry
    assert_eq!(
        config.default_font_path_by_lang("ja").unwrap(),
        ("/system/fonts/NotoSansCJK-Regular.ttc", 0)
    );
    assert_eq!(
        config.default_font_path_by_lang("zh-Hans").unwrap(),
        ("/system/fonts/NotoSansCJK-Regular.ttc", 2)
    );
    assert_eq!(
        config.default_font_path_by_lang("und-Khmr").unwrap(),
        ("/system/fonts/NotoSansKhmer-VF.ttf", 0)
    );
    assert_eq!(
        config.default_font_path_by_lang("und-Geor").unwrap(),
        ("/system/fonts/NotoSansGeorgian-Regular.otf", 0)
    );
    assert_eq!(
        config.default_font_path_by_lang("und-Geok").unwrap(),
        ("/system/fonts/NotoSansGeorgian-Regular.otf", 0)
    );
    assert_eq!(
        config.default_font_path_by_lang("und-Thai").unwrap(),
        ("/system/fonts/NotoSansThai-Regular.ttf", 0)
    );
}

#[cfg(test)]
#[test]
fn test_fallback_family_and_lang() {
    let config = AndroidFontConfig::new_from_file("data/fonts-1.xml");
    assert_eq!(
        config
            .font_path_by_family_and_lang("sans-serif", "und-Thai")
            .unwrap(),
        ("/system/fonts/NotoSansThai-Regular.ttf", 0)
    );
    assert_eq!(
        config
            .font_path_by_family_and_lang("serif", "und-Thai")
            .unwrap(),
        ("/system/fonts/NotoSerifThai-Regular.ttf", 0)
    );
    assert_eq!(
        config
            .font_path_by_family_and_lang("serif", "zh-Hans")
            .unwrap(),
        ("/system/fonts/NotoSerifCJK-Regular.ttc", 2)
    );
}

#[cfg(test)]
#[test]
fn test_all_font_paths() {
    let config = AndroidFontConfig::new_from_file("data/fonts-1.xml");
    assert!(config
        .all_font_paths()
        .contains(&("/system/fonts/NotoSansThai-Regular.ttf".to_owned(), 0)));
    assert!(config
        .all_font_paths()
        .contains(&("/system/fonts/Roboto-Thin.ttf".to_owned(), 0)));
    assert!(config
        .all_font_paths()
        .contains(&("/system/fonts/NotoSansCJK-Regular.ttc".to_owned(), 0)));
    assert!(config
        .all_font_paths()
        .contains(&("/system/fonts/NotoSansCJK-Regular.ttc".to_owned(), 2)));
}

#[cfg(test)]
#[test]
fn test_all_families() {
    let config = AndroidFontConfig::new_from_file("data/fonts-1.xml");
    assert!(config.all_families().contains(&"sans-serif".to_owned()));
    assert!(config.all_families().contains(&"serif".to_owned()));
    assert!(config.all_families().contains(&"monospace".to_owned()));
    assert!(config.all_families().contains(&"arial".to_owned()));
}
