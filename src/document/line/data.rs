
use std::{cmp, fmt, ops};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use derive_more::Display;
use crate::catalogue::CatalogueBuilder;
use crate::load::report::{Failed, Origin, PathReporter};
use crate::load::yaml::{FromYaml, Mapping, Value};
use crate::store::{FullStore, StoreLoader, XrefsBuilder};
use crate::types::list;
use crate::types::{
    CountryCode, Date, EventDate, IntoMarked, Key, LanguageCode, LanguageText,
    List, LocalText, Location, Marked, Set
};
use crate::document::{entity, point};
use crate::document::combined::{
    DocumentLink, LineLink, EntityLink, PathLink, PointLink,
    SourceLink
};
use crate::document::common::{
    Agreement, AgreementType, Basis, Common, Contract, Progress
};


//------------ Document ------------------------------------------------------

pub use crate::document::combined::LineDocument as Document;

impl<'a> Document<'a> {
    pub fn junctions(
        self, store: &'a FullStore
    ) -> impl Iterator<Item = point::Document<'a>> + 'a {
        self.data().points.iter_documents(store).filter(|doc| {
            doc.meta().junction
        })
    }

    pub fn first_junction_name(
        self, store: &'a FullStore, _lang: LanguageCode,
    ) -> &'a str {
        self.data().points.first_junction(store).data().name_in_jurisdiction(
            self.data().jurisdiction()
        )
    }

    pub fn last_junction_name(
        self, store: &'a FullStore, _lang: LanguageCode,
    ) -> &'a str {
        self.data().points.last_junction(store).data().name_in_jurisdiction(
            self.data().jurisdiction()
        )
    }

    pub fn title(self, lang: LanguageCode) -> Option<&'a str> {
        for event in &self.data().events {
            for record in &event.records {
                if let Some(name) = record.properties.name.as_ref() {
                    if let Some(name) = name.for_language(lang) {
                        return Some(name)
                    }
                }
            }
        }
        None
    }
}


//------------ Data ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Data {
    link: LineLink,

    pub common: Common,
    pub label: Set<Label>,
    pub note: Option<LanguageText>,
    pub current: Current,
    pub events: EventList,
    pub records: RecordList,
    pub points: Points,

    code: LineCode,
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

    pub fn code(&self) -> &LineCode {
        &self.code
    }

    pub fn current_status_at(&self, point: PointLink) -> Option<Status> {
        self.points.index_of(point).and_then(|idx| {
            match self.current.status.at_index(idx)? {
                Ok(status) => Some(*status),
                Err((left, right)) => Some(cmp::max(*left, *right)),
            }
        })
    }
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
            code: LineCode::from_key(common.key.as_value()),
            common,
            label: label?,
            note: note?,
            current: current?,
            events: events?,
            records: records?,
            points,
        })
    }

    pub fn xrefs(
        &self, 
        builder: &mut XrefsBuilder,
        _store: &crate::store::DataStore,
        _report: &mut crate::load::report::PathReporter,
    ) -> Result<(), Failed> {
        // points: line points
        for point in self.points.iter() {
            point.xrefs_mut(builder).lines.push(self.link);
        }

        // entity: line regions: Go over current and the events, find all the
        // regions and the longest section they apply to.
        let mut regions = HashMap::<entity::Link, Section>::new();
        for section in &self.current.region.sections {
            for region in &section.1 {
                regions.entry(
                    region.into_value()
                ).and_modify(|current| {
                    current.grow(&section.0)
                }).or_insert(section.0.clone());
            }
        }
        for event in &self.events {
            if let Some(region_list) = event.region() {
                let new_section = event.sections.overall(self.points.len());
                for region in region_list {
                    regions.entry(
                        region.into_value()
                    ).and_modify(|section| {
                        section.grow(&new_section)
                    }).or_insert(new_section.clone());
                }
            }
        }
        for (line, section) in regions {
            line.xrefs_mut(builder).line_regions.push((self.link, section));
        }
        Ok(())
    }

    pub fn catalogue(
        &self,
        builder: &mut CatalogueBuilder,
        _store: &FullStore,
        _report: &mut PathReporter,
    ) -> Result<(), Failed> {
        // Insert line.
        builder.catalogue_mut().lines.push(self.link);

        //--- Insert names.
        builder.insert_name(self.key().as_str().into(), self.link.into());
        builder.insert_name(self.code().as_str().into(), self.link.into());
        builder.insert_name(self.code().line().into(), self.link.into());
        let mut names = HashSet::new();
        for event in self.events.iter() {
            if let Some(some) = event.name() {
                for (_, name) in some {
                    names.insert(name.as_value());
                }
            }
        }
        for name in names {
            builder.insert_name(name.into(), self.link.into());
        }

        Ok(())
    }
}


//------------ LineCode ------------------------------------------------------

#[derive(Clone, Debug)]
pub struct LineCode {
    code: String,
    region_end: usize,
    line_start: usize,
}

impl LineCode {
    fn from_key(key: &Key) -> Self {
        let key = key.as_str();

        // Drop "line."
        let key = if key.starts_with("line.") {
            &key[5..]
        }
        else {
            return Self {
                code: key.into(),
                region_end: 0,
                line_start: 0,
            }
        };

        // Take off country code.
        let (country, mut key) = if key.as_bytes().get(2) == Some(&b'.') {
            (&key[0..2], &key[3..])
        }
        else {
            return Self {
                code: key.into(),
                region_end: 0,
                line_start: 0,
            }
        };

        let mut res = country.to_uppercase();
        let mut region_end = res.len();

        // Deal with countries that have regions. Currently, that’s only RU.
        if res == "RU" {
            if let Some((region, line)) = key.split_once('.') {
                res.push(' ');
                res.push_str(&region.to_uppercase());
                region_end = res.len();
                key = line;
            }
        }
        
        res.push(' ');
        let line_start = res.len();
        res.push_str(key);
        Self {
            code: res,
            region_end,
            line_start,
        }
    }

    pub fn as_str(&self) -> &str {
        self.code.as_str()
    }

    pub fn region(&self) -> &str {
        &self.code.as_str()[..self.region_end]
    }

    pub fn line(&self) -> &str {
        &self.code.as_str()[self.line_start..]
    }
}

impl PartialEq for LineCode {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Eq for LineCode { }

impl PartialOrd for LineCode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LineCode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.code.cmp(&other.code)
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

#[derive(Clone, Debug)]
pub struct Points {
    points: Vec<Marked<PointLink>>,
}

impl Points {
    pub fn iter_documents<'s>(
        &'s self, store: &'s FullStore
    ) -> impl Iterator<Item = point::Document<'s>> + DoubleEndedIterator + 's {
        self.points.iter().map(move |link| link.document(store))
    }

    pub fn first_point<'s>(
        &'s self, store: &'s FullStore
    ) -> point::Document<'s> {
        self.points.first().unwrap().document(store)
    }

    pub fn first_junction<'s>(
        &'s self, store: &'s FullStore
    ) -> point::Document<'s> {
        self.iter_documents(store).find(|point| {
            !point.data().is_never_junction()
        }).unwrap_or_else(|| self.first_point(store))
    }

    pub fn last_point<'s>(
        &'s self, store: &'s FullStore
    ) -> point::Document<'s> {
        self.points.last().unwrap().document(store)
    }

    pub fn last_junction<'s>(
        &'s self, store: &'s FullStore
    ) -> point::Document<'s> {
        self.iter_documents(store).rev().find(|point| {
            !point.data().is_never_junction()
        }).unwrap_or_else(|| self.last_point(store))
    }

    pub fn index_of(&self, point: PointLink) -> Option<usize> {
        self.points.iter().enumerate().find_map(|(idx, val)| {
            val.as_value().eq(&point).then(|| idx)
        })
    }

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

#[derive(Clone, Debug, Default)]
pub struct Current {
    pub category: CurrentValue<Set<Category>>,
    pub course: CurrentValue<List<CourseSegment>>,
    pub electrified: CurrentValue<Option<Set<Marked<Electrified>>>>,
    pub gauge: CurrentValue<Set<Gauge>>,
    pub goods: CurrentValue<Goods>,
    pub jurisdiction: CurrentValue<Marked<CountryCode>>,
    pub name: CurrentValue<Option<LocalText>>,
    pub operator: CurrentValue<Option<List<Marked<EntityLink>>>>,
    pub owner: CurrentValue<Option<List<Marked<EntityLink>>>>,
    pub passenger: CurrentValue<Passenger>,
    pub rails: CurrentValue<Marked<u8>>,
    pub region: CurrentValue<List<Marked<EntityLink>>>,
    pub reused: CurrentValue<Option<List<Marked<LineLink>>>>,
    pub status: CurrentValue<Status>,
    pub tracks: CurrentValue<Marked<u8>>,

    pub at_vzg: CurrentValue<Option<AtVzg>>,
    pub ch_bav: CurrentValue<Option<String>>,
    pub cz_sr72: CurrentValue<Option<String>>,
    pub de_vzg: CurrentValue<Option<DeVzg>>,
    pub fr_rfn: CurrentValue<Option<FrRfn>>,
    pub pl_id12: CurrentValue<Option<String>>,

    pub source: List<Marked<SourceLink>>,
    pub note: Option<LanguageText>,
}

impl Current {
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

        let at_vzg = value.take_default("at.VzG", context, report);
        let ch_bav = value.take_default("ch.BAV", context, report);
        let cz_sr72 = value.take_default("cz.SR72", context, report);
        let de_vzg = value.take_default("de.VzG", context, report);
        let fr_rfn = value.take_default("fr.RFN", context, report);
        let pl_id12 = value.take_default("pl.Id12", context, report);

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

            at_vzg: at_vzg?,
            ch_bav: ch_bav?,
            cz_sr72: cz_sr72?,
            de_vzg: de_vzg?,
            fr_rfn: fr_rfn?,
            pl_id12: pl_id12?,

            source: source?,
            note: note?,
        })
    }
}



//------------ CurrentValue --------------------------------------------------

#[derive(Clone, Debug)]
pub struct CurrentValue<T> {
    sections: List<(Section, T)>,
}

impl<T> CurrentValue<T> {
    /// Returns a reference to itself if it isn’t empty.
    pub fn and_then<'a, U, F>(&'a self, f: F) -> Option<U>
    where
        F: FnOnce(&'a Self) -> U
    {
        if self.sections.is_empty() {
            None
        }
        else {
            Some(f(self))
        }
    }

    pub fn as_slice(&self) -> &[(Section, T)] {
        self.sections.as_slice()
    }

    /// Returns the value at the given point index.
    ///
    /// If the index is a at section boundary, there may actually be two
    /// different values. This is returned via the `Some(Err(_))` variant.
    ///
    fn at_index(&self, idx: usize) -> Option<Result<&T, (&T, &T)>> {
        // Because we may need refs to two sections at the same time, we need
        // to work with windows of two.
        for window in self.sections.as_slice().windows(2) {
            let (one, two) = (&window[0], &window[1]);
            if one.0.start_idx > idx {
                continue
            }
            if one.0.end_idx > idx {
                return Some(Ok(&one.1))
            }
            else if one.0.end_idx == idx {
                return Some(Err((&one.1, &two.1)))
            }
        }
        // No window of two with the last item as the first element, so check
        // that now.
        let last = self.sections.last()?;
        if last.0.end_idx < idx {
            None
        }
        else {
            Some(Ok(&last.1))
        }
    }
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
        self.as_slice()
    }
}


//------------ EventList -----------------------------------------------------

#[derive(Clone, Debug, Default)]
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

#[derive(Clone, Debug)]
pub struct Event {
    pub date: EventDate,
    pub sections: SectionList,
    pub records: List<EventRecord>,
}

impl Event {
    pub fn document(&self) -> Option<&List<Marked<SourceLink>>> {
        self.prop(|record| record.document.as_ref())
    }

    pub fn name(&self) -> Option<&LocalText> {
        self.prop(|prop| prop.properties.name.as_ref())
    }

    pub fn region(&self) -> Option<&List<Marked<EntityLink>>> {
        self.prop(|prop| prop.properties.region.as_ref())
    }

    pub fn concession(&self) -> Option<&Concession> {
        self.prop(|prop| prop.concession.as_ref())
    }

    pub fn agreement(&self) -> Option<&Agreement> {
        self.prop(|prop| prop.agreement.as_ref())
    }

    fn prop<F: Fn(&EventRecord) -> Option<&T>, T>(
        &self, op: F
    ) -> Option<&T> {
        self.records.iter().find_map(|record| op(&record))
    }
}

impl Event {
    pub fn is_legal(&self) -> bool {
        self.concession().is_some()
        || self.agreement().is_some()
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
        let sections = SectionList::from_yaml(
            &mut value, point_context, report
        );

        let records = match value.take_opt("records", point_context, report) {
            Ok(Some(records)) => Ok(records),
            Ok(None) => {
                EventRecord::from_mapping(
                    &mut value, point_context, report
                ).map(List::with_value)
            }
            Err(err) => Err(err), 
        };

        value.exhausted(report)?;

        Ok(Event {
            date: date?,
            sections: sections?,
            records: records?,
        })
    }
}


//------------ EventRecord ---------------------------------------------------

#[derive(Clone, Debug)]
pub struct EventRecord {
    pub date: Option<EventDate>,
    pub document: Option<List<Marked<SourceLink>>>,
    pub source: Option<List<Marked<SourceLink>>>,
    pub basis: Option<List<Basis>>,
    pub note: Option<LanguageText>,

    pub concession: Option<Concession>,
    pub agreement: Option<Agreement>,

    pub properties: Properties,
}

impl EventRecord {
    fn from_mapping(
        value: &mut Mapping,
        point_context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let context = point_context.context;
        let date = value.take_opt("date", context, report);
        let document = value.take_opt("document", context, report);
        let source = value.take_opt("source", context, report);
        let basis = value.take_opt("basis", context, report);
        let note = value.take_opt("note", context, report);

        let concession = value.take_opt("concession", context, report);
        let expropriation = value.take_opt::<_, Concession>("expropriation", context, report);
        let agreement = value.take_opt("agreement", context, report);
        let contract: Result<Option<Contract>, _>
            = value.take_opt("contract", context, report);
        let treaty: Result<Option<Contract>, _>
            = value.take_opt("treaty", context, report);

        let properties = Properties::from_yaml(value, context, report);

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
        
        Ok(Self {
            date: date?,
            document: document?,
            source: source?,
            basis: basis?,
            note: note?,
            concession,
            agreement,
            properties: properties?,
        })
    }
}

impl FromYaml<PointsContext<'_>> for EventRecord {
    fn from_yaml(
        value: Value,
        point_context: &PointsContext,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let mut value = value.into_mapping(report)?;
        let res = Self::from_mapping(&mut value, point_context, report);
        value.exhausted(report)?;
        res
    }
}


//------------ RecordList ----------------------------------------------------

#[derive(Clone, Debug, Default)]
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

#[derive(Clone, Debug)]
pub struct Record {
    pub sections: SectionList,
    pub document: Marked<SourceLink>,
    pub note: Option<LanguageText>,

    pub properties: Properties
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


//------------ Properties ----------------------------------------------------

#[derive(Clone, Debug)]
pub struct Properties {
    pub category: Option<Set<Category>>,
    pub electrified: Option<Set<Marked<Electrified>>>,
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

    pub at_vzg: Option<AtVzg>,
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
        || self.at_vzg.is_some()
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

        let at_vzg = value.take_opt("at.VzG", context, report);
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

            at_vzg: at_vzg?,
            de_vzg: de_vzg?,
            fr_rfn: fr_rfn?,
        })
    }
}


//------------ SectionList ---------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
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

    /// Returns the maximum section covered by this event.
    fn overall(&self, len: usize) -> Section {
        if self.sections.is_empty() {
            return Section::all(len)
        }

        let mut iter = self.sections.iter();
        let mut res = iter.next().unwrap().clone();
        for section in iter {
            res.grow(section)
        }
        res
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

#[derive(Clone, Debug)]
pub struct Section {
    pub start: Option<Marked<PointLink>>,
    pub end: Option<Marked<PointLink>>,
    pub start_idx: usize,
    pub end_idx: usize,
}

impl Section {
    pub fn start_point<'s>(
        &'s self, line: &'s Data, store: &'s FullStore
    ) -> point::Document<'s> {
        match self.start {
            Some(link) => link.document(store),
            None => line.points.first_point(store),
        }
    }

    pub fn end_point<'s>(
        &'s self, line: &'s Data, store: &'s FullStore
    ) -> point::Document<'s> {
        match self.end {
            Some(link) => link.document(store),
            None => line.points.last_point(store),
        }
    }
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

    fn grow(&mut self, other: &Section) {
        if other.start_idx < self.start_idx {
            self.start = other.start;
            self.start_idx = other.start_idx;
        }
        if other.end_idx > self.end_idx {
            self.end = other.end;
            self.end_idx = other.end_idx;
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
    #[non_exhaustive]
    pub enum Category {
        { DeHauptbahn: "de.Hauptbahn" }
        { DeNebenbahn: "de.Nebenbahn" }
        { DeKleinbahn: "de.Kleinbahn" }
        { DeAnschl: "de.Anschl" }
        { DeBfgleis: "de.Bfgleis" }
        { DeStrab: "de.Strab" }

        { GbLight: "gb.Light" }
    }
}

impl Category {
    pub fn short_str(self) -> &'static str {
        if let Some(pos) = self.as_str().find('.') {
            &self.as_str()[pos + 1..]
        }
        else {
            self.as_str()
        }
    }
}


//------------ Concession ----------------------------------------------------

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Electrified {
    named: Option<&'static str>,
    generic: Option<GenericEl>,
}

impl Electrified {
    fn from_name(s: &str) -> Option<Self> {
        macro_rules! system {
            (
                $(
                    $name:expr => $sys:ident, $volt:expr, $acdc:ident;
                )*
            ) => {
                $(
                    if s == $name {
                        return Some(Electrified {
                            named: Some($name),
                            generic: Some(GenericEl::new(
                                ElSystem::$sys, $volt, AcDc::$acdc
                            )),
                        })
                    }
                )*
            }
        }

        system!(
            "at"      => Ole, 15000, Ac16;
            "be"      => Ole,  3000, Dc;
            "be.25"   => Ole, 25000, Ac50;
            "ch"      => Ole, 15000, Ac16;
            "ch.11k"  => Ole, 11000, Ac16;
            "cz.3"    => Ole,  3000, Dc;
            "cz.25"   => Ole, 25000, Ac50;
            "de"      => Ole, 15000, Ac16;
            "de.bln-1903"  => Rail,  550, Dc;
            "de.bln"  => Rail,  800, Dc;
            "de.hmb"  => Rail, 1200, Dc;
            "de.hmb-alt" => Ole, 6300, Ac25;
            "dk"      => Ole, 25000, Ac50;
            "gb.25"   => Ole, 25000, Ac50;
            "gb.rail" => Rail,  750, Dc;
            "fr.15"   => Ole,  1500, Dc;
            "fr.25"   => Ole, 25000, Ac50;
            "fr.lgv"  => Ole, 25000, Ac50;
            "hu"      => Ole, 25000, Ac50;
            "it.3"    => Ole,  3000, Dc;
            "it.25"   => Ole, 25000, Ac50;
            "lt"      => Ole, 25000, Ac50;
            "lu.25"   => Ole, 25000, Ac50;
            "nl"      => Ole,  1500, Dc;
            "nl.25"   => Ole, 25000, Ac50;
            "pl"      => Ole,  3000, Dc;
            "ru"      => Ole,  3000, Dc;
            "si"      => Ole,  3000, Dc;
        );
        None
    }

    pub fn named(&self) -> Option<&str> {
        self.named
    }

    pub fn generic(&self) -> Option<GenericEl> {
        self.generic
    }
}

impl FromStr for Electrified {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "none" {
            Ok(Self { named: None, generic: None })
        }
        else if let Some(res) = Self::from_name(s) {
            Ok(res)
        }
        else if let Ok(generic) = GenericEl::from_str(s) {
            Ok(Self {
                named: None,
                generic: Some(generic)
            })
        }
        else {
            Err(format!("unknown electrification system '{}'", s))
        }
    }
}

impl fmt::Display for Electrified {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = self.named {
            f.write_str(name)
        }
        else if let Some(generic) = self.generic {
            generic.fmt(f)
        }
        else {
            f.write_str("none")
        }
    }
}

impl FromYaml<StoreLoader> for Marked<Electrified> {
    fn from_yaml(
        value: Value,
        _context: &StoreLoader,
        report: &mut PathReporter
    ) -> Result<Self, Failed> {
        let text = value.into_string(report)?;
        let res = text.try_map(|plain| Electrified::from_str(&plain));
        res.map_err(|err| {
            report.error(err);
            Failed
        })
    }
}


//------------ GenericEl -----------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GenericEl {
    pub system: ElSystem,
    pub voltage: u16,
    pub frequency: AcDc
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ElSystem {
    Ole,
    Rail,
    Rail4,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AcDc {
    Ac16,
    Ac25,
    Ac50,
    Tc50,
    Dc,
}

impl GenericEl {
    const fn new(system: ElSystem, voltage: u16, frequency: AcDc) -> Self {
        Self { system, voltage, frequency }
    }
}

impl FromStr for GenericEl {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (system, s) = if let Some(s) = s.strip_prefix("ole.") {
            (ElSystem::Ole, s)
        }
        else if let Some(s) = s.strip_prefix("rail.") {
            (ElSystem::Rail, s)
        }
        else if let Some(s) = s.strip_prefix("rail4.") {
            (ElSystem::Rail4, s)
        }
        else {
            return Err("invalid generic electrification string")
        };

        let (frequency, s) = if let Some(s) = s.strip_suffix("ac16") {
            (AcDc::Ac16, s)
        }
        else if let Some(s) = s.strip_suffix("ac25") {
            (AcDc::Ac25, s)
        }
        else if let Some(s) = s.strip_suffix("ac50") {
            (AcDc::Ac50, s)
        }
        else if let Some(s) = s.strip_suffix("tc50") {
            (AcDc::Tc50, s)
        }
        else if let Some(s) = s.strip_suffix("dc") {
            (AcDc::Dc, s)
        }
        else {
            return Err("invalid generic electrification string")
        };

        let voltage = u16::from_str(s).map_err(|_| {
            "invalid generic electrification string"
        })?;

        Ok(Self { system, voltage, frequency })
    }
}

impl fmt::Display for GenericEl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.system {
            ElSystem::Ole => f.write_str("ole.")?,
            ElSystem::Rail => f.write_str("rail.")?,
            ElSystem::Rail4 => f.write_str("rail4.")?,
        }
        self.voltage.fmt(f)?;
        match self.frequency {
            AcDc::Ac16 => f.write_str("ac16"),
            AcDc::Ac25 => f.write_str("ac25"),
            AcDc::Ac50 => f.write_str("ac50"),
            AcDc::Tc50 => f.write_str("tc50"),
            AcDc::Dc => f.write_str("dc"),
        }
    }
}


//------------ Gauge ---------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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


//------------ Goods ---------------------------------------------------------

data_enum! {
    pub enum Goods {
        { None: "none" }
        { Limited: "limited" }
        { Full: "full" }
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


//------------ AtVzg ---------------------------------------------------------

pub type AtVzg = Marked<String>;


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


