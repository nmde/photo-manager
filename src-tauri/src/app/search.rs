use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use diesel::{
    debug_query, BoolExpressionMethods, ExpressionMethods, QueryDsl, TextExpressionMethods,
};
use diesel_async::RunQueryDsl;
use log::{debug, warn};
use strum::{Display, EnumString};

use crate::{
    app::{ensure_db, DATE_FORMAT, DB},
    models::{Person, Photo},
    schema::{people, photos},
};

#[derive(PartialEq)]
pub enum Sort {
    Date(bool),
    Name(bool),
    Rating(bool),
    FileDate(bool),
}

impl FromStr for Sort {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let sorting = if s.contains("_") {
            let mut split = s.split("_");
            let key = split.next();
            let dir = split.next();
            if key.is_none() || dir.is_none() {
                return Err(anyhow!("Invalid sorting: Expected format KEY_DIR"));
            }
            let key = key.unwrap().to_uppercase();
            (key, dir.unwrap().to_uppercase() == "DESC")
        } else {
            (s.to_uppercase(), false)
        };
        match sorting.0.as_str() {
            "DATE" => Ok(Sort::Date(sorting.1)),
            "NAME" => Ok(Sort::Name(sorting.1)),
            "RATING" => Ok(Sort::Rating(sorting.1)),
            "FILEDATE" => Ok(Sort::FileDate(sorting.1)),
            _ => Err(anyhow!("Unknown sort key: {}", sorting.0)),
        }
    }
}

impl Display for Sort {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Sort::Date(dir) => write!(f, "date{}", if *dir { " descending" } else { "" }),
            Sort::Name(dir) => write!(f, "name{}", if *dir { " descending" } else { "" }),
            Sort::Rating(dir) => write!(f, "rating{}", if *dir { " descending" } else { "" }),
            Sort::FileDate(dir) => write!(f, "file_date{}", if *dir { " descending" } else { "" }),
        }
    }
}

#[derive(Display, EnumString, PartialEq)]
enum HasTerm {
    Rating,
    Photographer,
    Date,
    Location,
    People,
    Tags,
}

#[derive(Display, EnumString, PartialEq)]
enum IsTerm {
    Raw,
    Video,
}

#[derive(Display, EnumString, PartialEq)]
enum CompOp {
    Eq,
    Ge,
    Le,
    Gt,
    Lt,
}

#[derive(PartialEq)]
enum SearchTerm {
    At(String),
    Only(String),
    By(String),
    Has(HasTerm),
    Name(String),
    Rating(CompOp, i32),
    Of(String),
    Date(CompOp, NaiveDate),
    Is(IsTerm),
    Tag(String),
    SortBy(Sort),
}

impl Display for SearchTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SearchTerm::At(loc) => write!(f, "AT({loc})"),
            SearchTerm::By(p) => write!(f, "BY({p})"),
            SearchTerm::Date(op, date) => write!(f, "DATE({op},{date})"),
            SearchTerm::Has(has) => write!(f, "HAS({has})"),
            SearchTerm::Is(is) => write!(f, "IS({is})"),
            SearchTerm::Name(name) => write!(f, "NAME({name})"),
            SearchTerm::Of(p) => write!(f, "OF({p})"),
            SearchTerm::Only(p) => write!(f, "ONLY({p})"),
            SearchTerm::Rating(op, r) => write!(f, "RATING({op},{r})"),
            SearchTerm::SortBy(sort) => write!(f, "SORTBY({sort})"),
            SearchTerm::Tag(tag) => write!(f, "TAG({tag})"),
        }
    }
}

fn parse_term(term: &str) -> Result<(SearchTerm, bool)> {
    let negated = term.starts_with('-');
    let t = if negated { &term[1..] } else { term };
    let up = t.to_uppercase();
    if up.starts_with("AT:") {
        Ok((SearchTerm::At(t[3..].to_string()), negated))
    } else if up.starts_with("ONLY:") {
        Ok((SearchTerm::Only(t[5..].to_string()), negated))
    } else if up.starts_with("BY:") {
        Ok((SearchTerm::By(t[3..].to_string()), negated))
    } else if up.starts_with("HAS:") {
        match &up[4..] {
            "RATING" => Ok((SearchTerm::Has(HasTerm::Rating), negated)),
            "PHOTOGRAPHER" => Ok((SearchTerm::Has(HasTerm::Photographer), negated)),
            "DATE" => Ok((SearchTerm::Has(HasTerm::Date), negated)),
            "LOCATION" => Ok((SearchTerm::Has(HasTerm::Location), negated)),
            "PEOPLE" => Ok((SearchTerm::Has(HasTerm::People), negated)),
            "TAGS" => Ok((SearchTerm::Has(HasTerm::Tags), negated)),
            s => Err(anyhow!("Unknown HAS: qualifier: {s}")),
        }
    } else if up.starts_with("NAME:") {
        Ok((SearchTerm::Name(t[5..].to_ascii_lowercase()), negated))
    } else if up.starts_with("RATING<=") {
        Ok((
            SearchTerm::Rating(CompOp::Le, t[8..].parse::<i32>()?),
            negated,
        ))
    } else if up.starts_with("RATING>=") {
        Ok((
            SearchTerm::Rating(CompOp::Ge, t[8..].parse::<i32>()?),
            negated,
        ))
    } else if up.starts_with("RATING<") {
        Ok((
            SearchTerm::Rating(CompOp::Lt, t[7..].parse::<i32>()?),
            negated,
        ))
    } else if up.starts_with("RATING>") {
        Ok((
            SearchTerm::Rating(CompOp::Gt, t[7..].parse::<i32>()?),
            negated,
        ))
    } else if up.starts_with("RATING=") {
        Ok((
            SearchTerm::Rating(CompOp::Eq, t[7..].parse::<i32>()?),
            negated,
        ))
    } else if up.starts_with("SORT:") || up.starts_with("SORT=") {
        Ok((SearchTerm::SortBy(Sort::from_str(&t[5..])?), negated))
    } else if up.starts_with("ORDER:") || up.starts_with("ORDER=") {
        Ok((SearchTerm::SortBy(Sort::from_str(&t[6..])?), negated))
    } else if up.starts_with("OF:") {
        let val = t[3..].to_string();
        Ok((SearchTerm::Of(val), negated))
    } else if up.starts_with("DATE>=") {
        Ok((
            SearchTerm::Date(CompOp::Ge, NaiveDate::parse_from_str(&t[6..], DATE_FORMAT)?),
            negated,
        ))
    } else if up.starts_with("DATE<=") {
        Ok((
            SearchTerm::Date(CompOp::Le, NaiveDate::parse_from_str(&t[6..], DATE_FORMAT)?),
            negated,
        ))
    } else if up.starts_with("DATE>") {
        Ok((
            SearchTerm::Date(CompOp::Gt, NaiveDate::parse_from_str(&t[5..], DATE_FORMAT)?),
            negated,
        ))
    } else if up.starts_with("DATE<") {
        Ok((
            SearchTerm::Date(CompOp::Lt, NaiveDate::parse_from_str(&t[5..], DATE_FORMAT)?),
            negated,
        ))
    } else if up.starts_with("DATE:") {
        Ok((
            SearchTerm::Date(CompOp::Eq, NaiveDate::parse_from_str(&t[5..], DATE_FORMAT)?),
            negated,
        ))
    } else if up.starts_with("IS:") {
        match &up[3..] {
            "VIDEO" => Ok((SearchTerm::Is(IsTerm::Video), negated)),
            "RAW" => Ok((SearchTerm::Is(IsTerm::Raw), negated)),
            s => Err(anyhow!("Unknown IS: qualifier: {s}")),
        }
    } else {
        Ok((SearchTerm::Tag(t.to_string()), negated))
    }
}

/// Performs a search of the photos using the given query.
pub async fn search_photos(query: &Vec<String>, sort: Sort) -> Result<Vec<Photo>> {
    debug!(
        "Searching photos with query \"{0}\", sorted by {1}",
        query.join(","),
        sort
    );
    let mut unmet_terms = vec![];
    let mut sort = sort;
    let mut terms = vec![];
    for term in query {
        let parsed = parse_term(term).with_context(|| "Could not parse search term: {term}")?;
        debug!(
            "Parsed term: {0}{1}",
            if parsed.1 { "NOT " } else { "" },
            parsed.0,
        );
        terms.push(parsed);
    }
    let needs_people = terms.iter().any(|term| matches!(term.0, SearchTerm::Of(_)));

    // Construct a SQL statement using terms that require no additional processing (is:..., at:..., only:..., by:..., has:...)
    let mut statement = photos::table
        .filter(
            photos::is_duplicate
                .eq(0)
                .or(photos::is_duplicate.is_null()),
        )
        .into_boxed();
    for term in terms {
        let negated = term.1;
        match term.0 {
            SearchTerm::At(location) => {
                if negated {
                    statement = statement.filter(photos::location.ne(location));
                } else {
                    statement = statement.filter(photos::location.eq(location));
                }
            }
            SearchTerm::Only(person) => {
                if negated {
                    statement = statement.filter(photos::people.ne(person));
                } else {
                    statement = statement.filter(photos::people.eq(person));
                }
            }
            SearchTerm::By(photographer) => {
                if negated {
                    statement = statement.filter(photos::photographer.ne(photographer));
                } else {
                    statement = statement.filter(photos::photographer.eq(photographer));
                }
            }
            SearchTerm::Has(has) => match has {
                HasTerm::Rating => {
                    if negated {
                        statement = statement.filter(photos::rating.is_null());
                    } else {
                        statement = statement.filter(photos::rating.is_not_null());
                    }
                }
                HasTerm::Photographer => {
                    if negated {
                        statement = statement.filter(photos::photographer.is_null());
                    } else {
                        statement = statement.filter(photos::photographer.is_not_null());
                    }
                }
                HasTerm::Date => {
                    if negated {
                        statement = statement.filter(photos::date.is_null());
                    } else {
                        statement = statement.filter(photos::date.is_not_null());
                    }
                }
                HasTerm::Location => {
                    if negated {
                        statement = statement.filter(photos::location.is_null());
                    } else {
                        statement = statement.filter(photos::location.is_not_null());
                    }
                }
                HasTerm::People => {
                    if negated {
                        statement = statement.filter(photos::people.is_null());
                    } else {
                        statement = statement.filter(photos::people.is_not_null());
                    }
                }
                HasTerm::Tags => {
                    if negated {
                        statement = statement.filter(photos::tags.is_null());
                    } else {
                        statement = statement.filter(photos::tags.is_not_null());
                    }
                }
            },
            SearchTerm::Name(name) => {
                if negated {
                    statement = statement.filter(photos::name.not_like(format!("%{name}%")));
                } else {
                    statement = statement.filter(photos::name.like(format!("%{name}%")));
                }
            }
            SearchTerm::Rating(op, rating) => match op {
                CompOp::Le => {
                    if negated {
                        statement = statement.filter(photos::rating.gt(rating));
                    } else {
                        statement = statement.filter(photos::rating.le(rating));
                    }
                }
                CompOp::Ge => {
                    if negated {
                        statement = statement.filter(photos::rating.lt(rating));
                    } else {
                        statement = statement.filter(photos::rating.ge(rating));
                    }
                }
                CompOp::Lt => {
                    if negated {
                        statement = statement.filter(photos::rating.ge(rating));
                    } else {
                        statement = statement.filter(photos::rating.lt(rating));
                    }
                }
                CompOp::Gt => {
                    if negated {
                        statement = statement.filter(photos::rating.le(rating));
                    } else {
                        statement = statement.filter(photos::rating.gt(rating));
                    }
                }
                CompOp::Eq => {
                    if negated {
                        statement = statement.filter(photos::rating.ne(rating));
                    } else {
                        statement = statement.filter(photos::rating.eq(rating));
                    }
                }
            },
            SearchTerm::SortBy(value) => {
                sort = value;
            }
            _ => {
                unmet_terms.push(term);
            }
        }
    }

    debug!(
        "Constructed SQL query for search: {}",
        debug_query(&statement)
    );

    ensure_db().await?;
    let mut conn = DB.lock().await;
    let conn = conn.as_mut().unwrap();
    let photo_records = statement.load::<Photo>(conn).await?;
    debug!("Query returned {} photos", photo_records.len());

    // Terms that require additional processing and iterating over the photos (date:..., of:..., any tags)
    let people = if needs_people {
        Some(
            people::table
                .load::<Person>(conn)
                .await?
                .into_iter()
                .map(|p| (p.id.clone(), p))
                .collect::<HashMap<String, Person>>(),
        )
    } else {
        None
    };

    let raw_name_map = photo_records
        .iter()
        .filter_map(|p| p.grouped_raw())
        .collect::<HashSet<String>>();

    let mut encountered_groups = HashSet::<String>::new();

    let mut results = photo_records
        .into_iter()
        .filter(|photo| {
            let photo_tags = photo.tags();
            let photo_people = photo.people();
            let photo_date = photo.date();

            if photo.is_raw() && raw_name_map.contains(&photo.name) {
                return false;
            }
            if photo.photo_group.is_some() {
                let group = photo.photo_group.as_ref().unwrap();
                if encountered_groups.contains(group) {
                    return false;
                }
            }

            let mut meets_terms = true;
            for term in &unmet_terms {
                let negated = term.1;
                meets_terms = meets_terms
                    && match &term.0 {
                        SearchTerm::Of(person) => {
                            let mut found = photo_people.contains(&person);
                            if !found {
                                let name = person.to_uppercase();
                                found = photo_people.iter().any(|id| {
                                    people
                                        .as_ref()
                                        .unwrap()
                                        .get(id)
                                        .map(|p| p.name.to_uppercase() == name)
                                        .unwrap_or(false)
                                });
                            }
                            found ^ negated
                        }
                        SearchTerm::Date(op, date) => {
                            if photo_date.is_some() {
                                let pd = photo_date.as_ref().unwrap();
                                (match op {
                                    CompOp::Eq => pd == date,
                                    CompOp::Ge => pd >= date,
                                    CompOp::Le => pd <= date,
                                    CompOp::Gt => pd > date,
                                    CompOp::Lt => pd < date,
                                }) ^ negated
                            } else {
                                false
                            }
                        }
                        SearchTerm::Is(is) => {
                            (match is {
                                IsTerm::Video => photo.is_video(),
                                IsTerm::Raw => photo.is_raw(),
                            }) ^ negated
                        }
                        SearchTerm::Tag(tag) => photo_tags.contains(&tag) ^ negated,
                        _ => {
                            warn!("Unexpected term: {}", term.0);
                            true
                        }
                    };
            }

            // Claim the group only after confirming this photo passes all terms,
            // so later photos in the same group can still be candidates if this one fails.
            if meets_terms {
                if let Some(group) = &photo.photo_group {
                    encountered_groups.insert(group.clone());
                }
            }

            meets_terms
        })
        .collect::<Vec<Photo>>();

    if match sort {
        Sort::Name(dir) => {
            results.sort_by(|a, b| a.name.cmp(&b.name));
            dir
        }
        Sort::Rating(dir) => {
            results.sort_by_cached_key(|p| p.rating);
            dir
        }
        Sort::Date(dir) => {
            results.sort_by_cached_key(|p| p.date());
            dir
        }
        Sort::FileDate(dir) => {
            results.sort_by_cached_key(|p| p.metadata_date());
            dir
        }
    } {
        results.reverse();
    }

    debug!("Search returned {} photos", results.len());
    Ok(results)
}
