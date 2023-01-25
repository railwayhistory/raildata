
use std::{fmt, ops};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::{StoreEnricher, StoreLoader};
use crate::types::list;
use crate::types::{
    CountryCode, Date, EventDate, IntoMarked, Key, LanguageCode, LanguageText,
    List, LocalText, Location, Marked, Set
};
use crate::document::combined::{
    DocumentLink, LineLink, EntityLink, PathLink, PointLink,
    SourceLink
};
use crate::document::common::{
    Agreement, AgreementType, Alternative, Basis, Common, Contract, Progress
};
use super::Meta;


//------------ Data ----------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Data {
    link: LineLink,
    pub common: Common,
    pub label: Set<Label>,
    pub note: Option<LanguageText>,
    pub current: Current,
    pub events: EventList,
    pub records: RecordList,
    pub points: Points,

    code: String
}

impl Data {
    pub fn link(&self) -> LineLink {
        self.link
    }

    pub fn key(&self) -> &Key {
        &self.common.key
    }

    pub fn progress(&self) -> Progress {
        self.common.progress.into_value()
    }

    pub fn origin(&self) -> &Origin {
        &self.common.origin
    }

    pub fn jurisdiction(&self) -> Option<CountryCode> {
        self.country()
    }

    pub fn country(&self) -> Option<CountryCode> {
        Self::country_from_key(self.key())
    }

    fn country_from_key(key: &Key) -> Option<CountryCode> {
        let key = key.as_str();
        if key.starts_with("line.") && key.get(7..8) == Some(".") {
            CountryCode::from_str(&key[5..7]).ok()
        }
        else {
            None
        }
    }

    pub fn name(&self, lang: LanguageCode) -> &str {
        for event in &self.events {
            if let Some(name) = event.name.as_ref() {
                if let Some(name) = name.for_language(lang) {
                    return name
                }
            }
        }
        self.code()
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    /*
    fn last_junction_index(&self, library: &Library) -> usize {
        self.points.iter().enumerate().rev().map(|(idx, point)| {
            (idx, point.follow(library))
        }).find_map(|(idx, point)| {
            if !point.is_never_junction() {
                Some(idx)
            }
            else {
                None
            }
        }).unwrap_or_else(|| self.points.len() - 1)
    }

    pub fn junctions<'a>(
        &'a self, library: &'a Library
    ) -> impl Iterator<Item=&'a Point> + 'a {
        let mut first = true;
        let last = self.last_junction_index(library);
        self.points.iter().enumerate().filter_map(move |(idx, point)| {
            let point = point.follow(library);
            if first {
                if !point.is_never_junction() {
                    first = false;
                    Some(point)
                }
                else {
                    None
                }
            }
            else if idx == last {
                Some(point)
            }
            else if idx > last {
                None
            }
            else if point.is_junction() {
                Some(point)
            }
            else {
                None
            }
        })
    }
    */
}

impl Data {
    pub fn from_yaml(
        key: Marked<Key>,
        mut doc: Mapping,
        link: DocumentLink,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let common = Common::from_yaml(key, &mut doc, context, report);
        let label = doc.take_default("label", context, report);
        let note = doc.take_opt("note", context, report);
        let points: Points = doc.take("points", context, report)?;
        let point_context = points.context(context);
        let current = doc.take_default("current", &point_context, report);
        let events = doc.take_default("events", &point_context, report);
        let records = doc.take_default("records", &point_context, report);
        doc.exhausted(report)?;

        let common = common?;

        Ok(Data {
            link: link.into(),
            code: Self::make_code(common.key.as_value()),
            common,
            label: label?,
            note: note?,
            current: current?,
            events: events?,
            records: records?,
            points,
        })
    }

    fn make_code(key: &Key) -> String {
        match Self::country_from_key(key) {
            Some(CountryCode::RU) => {
                if key.starts_with("line.ru.kg.") {
                    format!("RU КГ {}", &key[11..])
                }
                else {
                    format!("RU {} {}", &key[8..10], &key[11..])
                }
            }
            Some(country) => {
                format!("{} {}", country, &key[8..])
            }
            None => {
                format!("{}", &key[5..])
            }
        }
    }

    pub(crate) fn generate_meta(&self, store: &StoreEnricher) -> Meta {
        Meta::generate(self, store)
    }

/*
    pub fn verify(&self, report: &mut StageReporter) {
        verify::verify(self, report)
    }

    pub fn catalogue(
        &self,
        link: LineLink,
        catalogue: &mut Catalogue,
        _report: &mut StageReporter
    ) {
        let link = DocumentLink::from(link);
*/

    pub fn process_names<F: FnMut(String)>(&self, mut process: F) {
        let mut key = &self.key().as_str()[5..];
        while !key.is_empty() {
            process(key.into());
            match key.find('.') {
                Some(idx) => {
                    let (_, right) = key.split_at(idx);
                    key = &right[1..];
                }
                None => {
                    key = &""
                }
            }
        }

        let mut names = HashSet::new();
        for event in self.events.iter() {
            if let Some(some) = event.name.as_ref() {
                for (_, name) in some {
                    names.insert(name.as_value());
                }
            }
        }
        for name in names {
            process(name.into())
        }
    }
}


//------------ Label ---------------------------------------------------------

data_enum! {
    pub enum Label {
        { Connection: "connection" }
        { Goods: "goods" }
        { Port: "port" }
        { DeSBahn: "de.S-Bahn" }
    }
}


//------------ Points --------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Points {
    points: Vec<Marked<PointLink>>,
}

impl Points {
    fn context<'s>(&'s self, context: &'s StoreLoader) -> PointsContext<'s> {
        PointsContext {
            map: {
                self.points.iter().enumerate()
                    .map(|(index, link)| (link.into_value(), index))
                    .collect()
            },
            len: self.points.len(),
            context
        }
    }
}

impl FromYaml<StoreLoader> for Points {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let pos = value.location();
        let points = Vec::from_yaml(
            value, context, report
        )?;
        if points.is_empty() {
            report.error(
                EmptyPoints.marked(pos)
            );
            Err(Failed)
        }
        else if points.len() == 1 {
            report.error(
                SingleItemInPoints.marked(pos)
            );
            Err(Failed)
        }
        else {
            Ok(Points { points })
        }
    }
}

impl ops::Deref for Points {
    type Target = [Marked<PointLink>];

    fn deref(&self) -> &Self::Target {
        self.points.as_ref()
    }
}

impl<I: std::slice::SliceIndex<[Marked<PointLink>]>> ops::Index<I> for Points {
    type Output = <I as std::slice::SliceIndex<[Marked<PointLink>]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.points[index]
    }
}


//------------ PointsContext -------------------------------------------------

#[derive(Clone, Debug)]
struct PointsContext<'a> {
    map: HashMap<PointLink, usize>,
    len: usize,
    context: &'a StoreLoader,
}

impl PointsContext<'_> {
    fn get_index(&self, link: PointLink) -> Option<usize> {
        self.map.get(&link).cloned()
    }
}

impl<'a> ops::Deref for PointsContext<'a> {
    type Target = StoreLoader;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}


//------------ Current -------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Default, Serialize)]
pub struct Current {
    pub category: CurrentValue<Set<Category>>,
    pub course: CurrentValue<List<CourseSegment>>,
    pub electrified: CurrentValue<Option<Electrified>>,
    pub gauge: CurrentValue<Set<Gauge>>,
    pub goods: CurrentValue<Goods>,
    pub jurisdiction: CurrentValue<Marked<CountryCode>>,
    pub name: CurrentValue<LocalText>,
    pub operator: CurrentValue<Option<List<Marked<EntityLink>>>>,
    pub owner: CurrentValue<Option<List<Marked<EntityLink>>>>,
    pub passenger: CurrentValue<Passenger>,
    pub rails: CurrentValue<Marked<u8>>,
    pub region: CurrentValue<List<Marked<EntityLink>>>,
    pub reused: CurrentValue<Option<List<Marked<LineLink>>>>,
    pub status: CurrentValue<Status>,
    pub tracks: CurrentValue<Marked<u8>>,

    pub de_vzg: CurrentValue<Option<DeVzg>>,
    pub fr_rfn: CurrentValue<Option<FrRfn>>,

    pub source: List<Marked<SourceLink>>,
    pub note: Option<LanguageText>,
}

impl FromYaml<PointsContext<'_>> for Current {
    fn from_yaml(
        value: Value,
        context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        
        let category = value.take_default("category", context, report);
        let course = value.take_default("course", context, report);
        let electrified = value.take_default("electrified", context, report);
        let gauge = value.take_default("gauge", context, report);
        let goods = value.take_default("goods", context, report);
        let jurisdiction = value.take_default("jurisdiction", context, report);
        let name = value.take_default("name", context, report);
        let operator = value.take_default("operator", context, report);
        let owner = value.take_default("owner", context, report);
        let passenger = value.take_default("passenger", context, report);
        let rails = value.take_default("rails", context, report);
        let region = value.take_default("region", context, report);
        let reused = value.take_default("reused", context, report);
        let status = value.take_default("status", context, report);
        let tracks = value.take_default("tracks", context, report);

        let de_vzg = value.take_default("de.VzG", context, report);
        let fr_rfn = value.take_default("fr.RFN", context, report);

        let source = value.take_default("source", context.context, report);
        let note = value.take_opt("note", context, report);

        value.exhausted(report)?;
        
        Ok(Current {
            category: category?,
            course: course?,
            electrified: electrified?,
            gauge: gauge?,
            goods: goods?,
            jurisdiction: jurisdiction?,
            name: name?,
            operator: operator?,
            owner: owner?,
            passenger: passenger?,
            rails: rails?,
            region: region?,
            reused: reused?,
            status: status?,
            tracks: tracks?,

            de_vzg: de_vzg?,
            fr_rfn: fr_rfn?,

            source: source?,
            note: note?,
        })
    }
}



//------------ CurrentValue --------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct CurrentValue<T> {
    sections: List<(Section, T)>,
}

impl<T> Default for CurrentValue<T> {
    fn default() -> Self {
        CurrentValue { sections: List::default() }
    }
}

impl<T> FromYaml<PointsContext<'_>> for CurrentValue<T>
where T: FromYaml<StoreLoader> {
    fn from_yaml(
        value: Value,
        context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let pos = value.location();
        match value.try_into_mapping() {
            Ok(value) => {
                let mut ok = true;
                let mut sections = List::new();
                let mut start = None;
                let mut start_idx = 0;
                for (key, value) in value.into_iter() {
                    let end = match Marked::from_string(key, report) {
                        Ok(key) => key,
                        Err(_) => {
                            ok = false;
                            continue;
                        }
                    };
                    let end = PointLink::build(end, context, report);
                    let end_idx = match context.get_index(*end) {
                        Some(idx) => idx,
                        None => {
                            report.error(
                                PointNotListed.marked(end.location())
                            );
                            ok = false;
                            continue;
                        }
                    };
                    if end_idx <= start_idx {
                        report.error(
                            EndBeforeStart.marked(end.location())
                        );
                        ok = false;
                        continue;
                    }
                    let value = match T::from_yaml(value, context, report) {
                        Ok(value) => value,
                        Err(_) => {
                            ok = false;
                            continue;
                        }
                    };
                    let end = if end_idx == context.len - 1 {
                        None
                    }
                    else {
                        Some(end)
                    };
                    sections.push((
                        Section::new(start, end, start_idx, end_idx),
                        value
                    ));
                    start = end;
                    start_idx = end_idx;

                }
                if let Some(last) = sections.last() {
                    if last.0.end.is_some() {
                        report.error(ShortCurrentValue.marked(pos));
                        ok = false;
                    }
                }
                if ok {
                    Ok(CurrentValue { sections })
                }
                else {
                    Err(Failed)
                }
            }
            Err(value) => {
                T::from_yaml(value, context, report).map(|value| {
                    CurrentValue {
                        sections: List::with_value(
                            (Section::all(context.len), value)
                        )
                    }
                })
            }
        }
    }
}

impl<T> ops::Deref for CurrentValue<T> {
    type Target = [(Section, T)];
    
    fn deref(&self) -> &Self::Target {
        self.sections.as_slice()
    }
}


//------------ EventList -----------------------------------------------------

#[derive(Clone, Deserialize, Debug, Default, Serialize)]
pub struct EventList {
    events: List<Event>
}

impl FromYaml<PointsContext<'_>> for EventList {
    fn from_yaml(
        value: Value,
        context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut res = List::from_yaml(value, context, report).map(|events| {
            EventList { events }
        })?;
        res.events.sort_by(|left, right| left.date.sort_cmp(&right.date));
        Ok(res)
    }
}


impl ops::Deref for EventList {
    type Target = List<Event>;

    fn deref(&self) -> &Self::Target {
        &self.events
    }
}


impl<'a> IntoIterator for &'a EventList {
    type Item = &'a Event;
    type IntoIter = list::Iter<'a, Event>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.iter()
    }
}


//------------ Event ---------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Event {
    pub date: EventDate,
    pub sections: SectionList,
    pub document: List<Marked<SourceLink>>,
    pub source: List<Marked<SourceLink>>,
    pub alternative: List<Alternative>,
    pub basis: List<Basis>,
    pub note: Option<LanguageText>,

    pub concession: Option<Concession>,
    pub agreement: Option<Agreement>,

    properties: Properties,
}

impl Event {
    pub fn is_legal(&self) -> bool {
        self.concession.is_some()
        || self.agreement.is_some()
    }
}

impl FromYaml<PointsContext<'_>> for Event {
    fn from_yaml(
        value: Value,
        point_context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let context = point_context.context;
        let mut value = value.into_mapping(report)?;
        let date = value.take_default("date", context, report);
        let document = value.take_default("document", context, report);
        let source = value.take_default("source", context, report);
        let alternative = value.take_default("alternative", context, report);
        let basis = value.take_default("basis", context, report);
        let note = value.take_opt("note", context, report);

        let concession = value.take_opt("concession", context, report);
        let expropriation = value.take_opt::<_, Concession>("expropriation", context, report);
        let agreement = value.take_opt("agreement", context, report);
        let contract: Result<Option<Contract>, _>
            = value.take_opt("contract", context, report);
        let treaty: Result<Option<Contract>, _>
            = value.take_opt("treaty", context, report);

        let sections = SectionList::from_yaml(
            &mut value, point_context, report
        );
        let properties = Properties::from_yaml(&mut value, context, report);

        value.exhausted(report)?;

        let concession = concession?;
        let expropriation = expropriation?;
        let concession = match (concession, expropriation) {
            (None, None) => None,
            (Some(concession), None) => Some(concession),
            (None, Some(mut concession)) => {
                let pos = concession.pos;
                concession.rights = Set::one(
                    ConcessionRight::Expropriation.marked(pos), pos
                );
                Some(concession)
            }
            (Some(_), Some(expro)) => {
                report.error(MultipleConcessions.marked(expro.pos));
                return Err(Failed)
            }
        };

        let agreement = agreement?;
        let contract = contract?;
        let treaty = treaty?;

        let agreement = if let Some(agreement) = agreement {
            if let Some(contract) = contract {
                report.error(MultipleAgreements.marked(contract.pos));
                return Err(Failed)
            }
            if let Some(treaty) = treaty {
                report.error(MultipleAgreements.marked(treaty.pos));
                return Err(Failed)
            }
            Some(agreement)
        }
        else if let Some(contract) = contract {
            if let Some(treaty) = treaty {
                report.error(MultipleAgreements.marked(treaty.pos));
                return Err(Failed)
            }
            Some(contract.into_agreement(AgreementType::Contract))
        }
        else if let Some(treaty) = treaty {
            Some(treaty.into_agreement(AgreementType::Treaty))
        }
        else {
            None
        };
        
        Ok(Event {
            date: date?,
            sections: sections?,
            document: document?,
            source: source?,
            alternative: alternative?,
            basis: basis?,
            note: note?,

            concession,
            agreement,

            properties: properties?,
        })
    }
}

impl ops::Deref for Event {
    type Target = Properties;

    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}


//------------ RecordList ----------------------------------------------------

#[derive(Clone, Deserialize, Debug, Default, Serialize)]
pub struct RecordList {
    documents: Vec<(SourceLink, List<Record>)>,
}

impl RecordList {
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    pub fn documents(
        &self
    ) -> impl Iterator<Item = (SourceLink, &List<Record>)> {
        self.documents.iter().map(|(link, list)| (*link, list))
    }
}

impl FromYaml<PointsContext<'_>> for RecordList {
    fn from_yaml(
        value: Value,
        context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut list = Vec::<Record>::from_yaml(value, context, report)?;
        list.sort_by(|left, right| left.sections.cmp(&right.sections));
        let mut map: HashMap<SourceLink, List<Record>> = HashMap::default();
        for record in list {
            map.entry(record.document.into_value()).or_default().push(record)
        }
        Ok(RecordList {
            documents: map.into_iter().collect()
        })
    }
}


//------------ Record --------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Record {
    pub sections: SectionList,
    pub document: Marked<SourceLink>,
    pub note: Option<LanguageText>,

    properties: Properties
}

impl FromYaml<PointsContext<'_>> for Record {
    fn from_yaml(
        value: Value,
        context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let date = value.take_default("date", context.context, report);
        let document = value.take("document", context.context, report);
        let note = value.take_opt("note", context.context, report);
        let sections = SectionList::from_yaml(&mut value, context, report);
        let properties = Properties::from_yaml(
            &mut value, context.context, report
        );
        value.exhausted(report)?;

        let _: EventDate = date?;

        Ok(Record {
            sections: sections?,
            document: document?,
            note: note?,
            properties: properties?,
        })
    }
}

impl ops::Deref for Record {
    type Target = Properties;

    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}


//------------ Properties ----------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Properties {
    pub category: Option<Set<Category>>,
    pub electrified: Option<Option<Set<Electrified>>>,
    pub gauge: Option<Set<Gauge>>,
    pub name: Option<LocalText>,
    pub rails: Option<Marked<u8>>,
    pub reused: Option<List<Marked<LineLink>>>,
    pub status: Option<Status>,
    pub tracks: Option<Marked<u8>>,

    pub goods: Option<Goods>,
    pub passenger: Option<Passenger>,

    pub constructor: Option<List<Marked<EntityLink>>>,
    pub operator: Option<List<Marked<EntityLink>>>,
    pub owner: Option<List<Marked<EntityLink>>>,
    pub jurisdiction: Option<Marked<CountryCode>>,

    pub course: Option<List<CourseSegment>>,
    pub region: Option<List<Marked<EntityLink>>>,

    pub de_vzg: Option<DeVzg>,
    pub fr_rfn: Option<FrRfn>,
}

impl Properties {
    pub fn has_properties(&self) -> bool {
        self.category.is_some()
        || self.constructor.is_some()
      //|| self.course.is_some()
        || self.electrified.is_some()
        || self.goods.is_some()
        || self.gauge.is_some()
      //|| self.jurisdiction.is_some()
        || self.name.is_some()
        || self.operator.is_some()
        || self.owner.is_some()
        || self.passenger.is_some()
        || self.rails.is_some()
      //|| self.region.is_some()
      //|| self.reused.is_some()
        || self.status.is_some()
        || self.tracks.is_some()
        || self.de_vzg.is_some()
        || self.fr_rfn.is_some()
    }
}

impl Properties {
    fn from_yaml(
        value: &mut Mapping,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let category = value.take_opt("category", context, report);
        let constructor = value.take_opt("constructor", context, report);
        let course = value.take_default("course", context, report);
        let electrified = value.take_opt("electrified", context, report);
        let goods = value.take_opt("goods", context, report);
        let gauge = value.take_opt("gauge", context, report);
        let jurisdiction = value.take_opt("jurisdiction", context, report);
        let name = value.take_opt("name", context, report);
        let operator = value.take_opt("operator", context, report);
        let owner = value.take_opt("owner", context, report);
        let passenger = value.take_opt("passenger", context, report);
        let rails = value.take_opt("rails", context, report);
        let region = value.take_opt("region", context, report);
        let reused = value.take_opt("reused", context, report);
        let status = value.take_opt("status", context, report);
        let tracks = value.take_opt("tracks", context, report);

        let de_vzg = value.take_opt("de.VzG", context, report);
        let fr_rfn = value.take_opt("fr.RFN", context, report);
        
        Ok(Properties {
            category: category?,
            constructor: constructor?,
            course: course?,
            electrified: electrified?,
            goods: goods?,
            gauge: gauge?,
            jurisdiction: jurisdiction?,
            name: name?,
            operator: operator?,
            owner: owner?,
            passenger: passenger?,
            rails: rails?,
            region: region?,
            reused: reused?,
            status: status?,
            tracks: tracks?,

            de_vzg: de_vzg?,
            fr_rfn: fr_rfn?,
        })
    }
}


//------------ SectionList ---------------------------------------------------

#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Serialize)]
pub struct SectionList {
    sections: List<Section>,
}

impl SectionList {
    fn from_yaml(
        value: &mut Mapping,
        context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let sections = value.take_default("sections", context, report);
        let start = value.take_opt("start", context.context, report);
        let end = value.take_opt("end", context.context, report);

        let mut sections: List<Section> = sections?;
        let start: Option<Marked<PointLink>> = start?;
        let end: Option<Marked<PointLink>> = end?;
        match (start, end) {
            (None, None) => {
                if sections.is_empty() {
                    sections.push(Section::all(context.len))
                }
            },
            (start, end) => {
                if !sections.is_empty() {
                    if let Some(start) = start {
                        report.error(
                            StartWithSections.marked(start.location())
                        );
                    }
                    if let Some(end) = end {
                        report.error(EndWithSections.marked(end.location()));
                    }
                    return Err(Failed)
                }
                sections.push(
                    Section::build(start, end, context, report)?
                )
            }
        };

        Ok(SectionList { sections })
    }
}

impl ops::Deref for SectionList {
    type Target = List<Section>;

    fn deref(&self) -> &Self::Target {
        &self.sections
    }
}

impl PartialOrd for SectionList {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SectionList {
    fn cmp(&self, other: &Self) -> Ordering {
        self.first().cmp(&other.first())
    }
}

impl<'a> IntoIterator for &'a SectionList {
    type Item = &'a Section;
    type IntoIter = list::Iter<'a, Section>;

    fn into_iter(self) -> Self::IntoIter {
        self.sections.iter()
    }
}


//------------ Section -------------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Section {
    pub start: Option<Marked<PointLink>>,
    pub end: Option<Marked<PointLink>>,
    pub start_idx: usize,
    pub end_idx: usize,
}

impl Section {
    fn new(
        start: Option<Marked<PointLink>>,
        end: Option<Marked<PointLink>>,
        start_idx: usize,
        end_idx: usize
    ) -> Self {
        Section { start, end, start_idx, end_idx }
    }

    fn build(
        start: Option<Marked<PointLink>>,
        end: Option<Marked<PointLink>>,
        context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let start_idx = match start {
            Some(start) => match context.get_index(*start) {
                Some(idx) => Ok(idx),
                None => {
                    report.error(StartNotListed.marked(start.location()));
                    Err(Failed)
                }
            }
            None => Ok(0),
        };
        let end_idx = match end {
            Some(end) => match context.get_index(*end) {
                Some(idx) => Ok(idx),
                None => {
                    report.error(EndNotListed.marked(end.location()));
                    Err(Failed)
                }
            }
            None => Ok(context.len - 1)
        };
        let start_idx = start_idx?;
        let end_idx = end_idx?;
        let location = if let Some(start) = start {
            start.location()
        }
        else if let Some(end) = end {
            end.location()
        }
        else {
            Location::default()
        };
        if start_idx == end_idx {
            report.error(EmptySection.marked(location));
            Err(Failed)
        }
        else if start_idx > end_idx {
            report.error(EndBeforeStart.marked(location));
            Err(Failed)
        }
        else {
            Ok(Section {
                start: start,
                end: end,
                start_idx,
                end_idx,
            })
        }
    }

    fn all(len: usize) -> Self {
        Section {
            start: None,
            end: None,
            start_idx: 0,
            end_idx: len - 1,
        }
    }
}


impl FromYaml<PointsContext<'_>> for Section {
    fn from_yaml(
        value: Value,
        context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let start = value.take_opt("start", context.context, report);
        let end = value.take_opt("end", context.context, report);
        value.exhausted(report)?;
        Self::build(start?, end?, context, report)
    }
}

impl PartialEq for Section {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl Eq for Section { }

impl PartialOrd for Section {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Section {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.start_idx.cmp(&other.start_idx) {
            Ordering::Equal => self.end_idx.cmp(&other.end_idx),
            res => res
        }
    }
}


//------------ Category ------------------------------------------------------

data_enum! {
    pub enum Category {
        { DeHauptbahn: "de.Hauptbahn" }
        { DeNebenbahn: "de.Nebenbahn" }
        { DeKleinbahn: "de.Kleinbahn" }
        { DeAnschl: "de.Anschl" }
        { DeBfgleis: "de.Bfgleis" }
        { DeStrab: "de.Strab" }
    }
}


//------------ Concession ----------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Concession {
    pub by: List<Marked<EntityLink>>,
    pub to: List<Marked<EntityLink>>,
    pub rights: Set<Marked<ConcessionRight>>,
    pub until: Option<Marked<Date>>,
    pub pos: Location,
}


impl FromYaml<StoreLoader> for Concession {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let pos = value.location();
        let mut value = value.into_mapping(report)?;
        let by = value.take_default("by", context, report);
        let to = value.take_default("for", context, report);
        let rights = value.take_default("rights", context, report);
        let until = value.take_opt("until", context, report);
        value.exhausted(report)?;
        Ok(Concession {
            by: by?,
            to: to?,
            rights: rights?,
            until: until?,
            pos
        })
    }
}


//------------ ConcessionRight -----------------------------------------------

data_enum! {
    pub enum ConcessionRight {
        { Construction: "construction" }
        { Operation: "operation" }
        { Expropriation: "expropriation" }
    }
}


//------------ CourseSegment -------------------------------------------------

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct CourseSegment {
    pub path: Marked<PathLink>,
    pub start: Marked<String>,
    pub end: Marked<String>,
}

impl FromYaml<StoreLoader> for CourseSegment {
    fn from_yaml(
        value: Value,
        context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let (value, location) = value.into_string(report)?.unwrap();
        let mut value = value.split_whitespace();
        let path = match value.next() {
            Some(path) => path,
            None => {
                report.error(InvalidCourseSegment.marked(location));
                return Err(Failed)
            }
        };
        let key = match Key::from_str(path) {
            Ok(key) => key.marked(location),
            Err(err) => {
                report.error(err.marked(location));
                return Err(Failed)
            }
        };
        let path = PathLink::build(key, context, report);
        let start = match value.next() {
            Some(path) => path,
            None => {
                report.error(InvalidCourseSegment.marked(location));
                return Err(Failed)
            }
        };
        let start = Marked::new(String::from(start), location);
        let end = match value.next() {
            Some(path) => path,
            None => {
                report.error(InvalidCourseSegment.marked(location));
                return Err(Failed)
            }
        };
        let end = Marked::new(String::from(end), location);
        if value.next().is_some() {
            report.error(InvalidCourseSegment.marked(location));
            return Err(Failed)
        }
        Ok(CourseSegment { path, start, end })
    }
}

impl PartialEq for CourseSegment {
    fn eq(&self, other: &Self) -> bool {
        self.path.as_value() == other.path.as_value()
        && self.start.as_value() == other.start.as_value()
        && self.end.as_value() == other.start.as_value()
    }
}


//------------ Electrified ---------------------------------------------------

pub type Electrified = Marked<String>;


//------------ Goods ---------------------------------------------------------

data_enum! {
    pub enum Goods {
        { None: "none" }
        { Limited: "limited" }
        { Full: "full" }
    }
}


//------------ Gauge ---------------------------------------------------------

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd,
    Serialize
)]
pub struct Gauge(pub Marked<u16>);

impl Gauge {
    pub fn gauge(&self) -> u16 {
        self.0.to_value()
    }

    pub fn location(&self) -> Location {
        self.0.location()
    }
}

impl Default for Gauge {
    fn default() -> Gauge {
        Gauge(Marked::from_value(1435))
    }
}

impl<C> FromYaml<C> for Gauge {
    fn from_yaml(
        value: Value,
        _: &C,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let (value, location) = value.into_string(report)?.unwrap();
        if !value.ends_with("mm") {
            report.error(InvalidGauge.marked(location));
            return Err(Failed)
        }
        match u16::from_str(&value[0..value.len() - 2]) {
            Ok(value) => Ok(Gauge(Marked::new(value, location))),
            Err(_) => {
                report.error(InvalidGauge.marked(location));
                Err(Failed)
            }
        }
    }
}

impl fmt::Display for Gauge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\u{202f}mm", self.0)
    }
}


//------------ Passenger -----------------------------------------------------

data_enum! {
    pub enum Passenger {
        { None: "none" }
        { Limited: "limited" }
        { Historic: "historic" }
        { Seasonal: "seasonal" }
        { Tourist: "tourist" }
        { Full: "full" }
    }
}


//------------ Status --------------------------------------------------------

data_enum! {
    pub enum Status {
        { None: "none" }
        { Planned: "planned" }
        { Construction: "construction" }
        { Open: "open" }
        { Suspended: "suspended" }
        { Reopened: "reopened" }
        { Closed: "closed" }
        { Removed: "removed" }
        { Released: "released" }
    }
}


//------------ DeVzg ---------------------------------------------------------

pub type DeVzg = Marked<String>;


//------------ FrRfn ---------------------------------------------------------

pub type FrRfn = Marked<String>;


//============ Errors ========================================================

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="empty list of points")]
pub struct EmptyPoints; 

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="points must contain at least two items")]
pub struct SingleItemInPoints; 

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="current value does not go all the way")]
pub struct ShortCurrentValue;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="start attribute not allowed when sections is present")]
pub struct StartWithSections;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="end attribute not allowed when sections is present")]
pub struct EndWithSections;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="point not in point list")]
pub struct PointNotListed; 

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt = "start point not in point list")]
pub struct StartNotListed;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt = "end point not in point list")]
pub struct EndNotListed;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt = "start and end are equal")]
pub struct EmptySection;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt = "end before start")]
pub struct EndBeforeStart;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="invalid gauge (must be an integer followed by 'mm'")]
pub struct InvalidGauge;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="invalid course segment")]
pub struct InvalidCourseSegment;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="only one of 'agreement', 'contract', or 'treaty' allowed")]
pub struct MultipleAgreements;

#[derive(Clone, Copy, Debug, Display)]
#[display(fmt="only one of 'concession' or 'expropriation' allowed")]
pub struct MultipleConcessions;


