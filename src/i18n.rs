//! Todos los textos que ve una persona, en un solo sitio.
//!
//! Español por defecto, ingles preparado. Ningún modulo escribe frases sueltas:
//! cuando un texto lleva datos, se compone aquí con una función.

use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    Es,
    En,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Self::Es => "es",
            Self::En => "en",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    AppName,
    TabHome,
    TabPreview,
    TabOrganize,
    TabHistory,
    TabSettings,
    OrganizeNow,
    Undo,
    Rescan,
    ChooseFolder,
    AddFolder,
    Remove,
    WizardStep,
    WizardTitle,
    WizardBody,
    WizardNext,
    WizardBack,
    WizardLast,
    Watching,
    LastOrganizeNever,
    LastOrganize,
    Scanning,
    RecentActivity,
    SeeAll,
    ColTime,
    ColAction,
    ColFiles,
    ColDestination,
    TotalOrganized,
    SpaceFreed,
    ActiveRules,
    PreviewTitle,
    PreviewSummary,
    PreviewEmpty,
    PreviewSkipped,
    PreviewFilter,
    PreviewSelectAll,
    MovedNotice,
    Category,
    Extensions,
    AddExtension,
    ExtensionPlaceholder,
    SubfoldersTitle,
    SubfoldersLabel,
    SubByDate,
    SubByDateHint,
    SubByType,
    SubByTypeHint,
    SubByCapture,
    SubByCaptureHint,
    GeneralTitle,
    FoldersTitle,
    RulesTitle,
    WatchTitle,
    AppearanceTitle,
    AdvancedTitle,
    ThemeLabel,
    ThemeDark,
    ThemeLight,
    ThemeSystem,
    LanguageLabel,
    SilentMode,
    NotificationsTitle,
    SilentOn,
    SilentOff,
    AdvancedMode,
    AdvancedHint,
    SilenceBar,
    ScheduleLabel,
    ScheduleOff,
    DebounceLabel,
    RecentGuardLabel,
    ExcludeLabel,
    AccentLabel,
    VersionLabel,
    CustomRules,
    RuleUp,
    RuleDown,
    RuleDelete,
    RuleAdd,
    RuleName,
    RuleKind,
    RuleValue,
    RuleAction,
    RuleTarget,
    KindExtension,
    KindGlob,
    KindMinSize,
    KindOlder,
    ActionMove,
    ActionTrash,
    ActionRename,
    ActionRenameUnavailable,
    HistoryTitle,
    HistoryEmptyTitle,
    HistoryEmpty,
    HistoryUndo,
    ExportCsv,
    ConfirmTrashTitle,
    ConfirmTrashBody,
    ConfirmMoveTitle,
    ConfirmMoveBody,
    Cancel,
    ConfirmTrashYes,
    ConfirmMoveYes,
    EmptyNoRoots,
    EmptyNoPlan,
    ErrorTitle,
    Dismiss,
    BatchLabel,
    FilesLabel,
    ItemsLabel,
    UndoDone,
    UndoNothing,
    UndoPartial,
    NothingToUndo,
    ExportDone,
    ExportEmpty,
    ScheduleMinutes,
    DaysLabel,
    SizeLabel,
    BytesLabel,
    FolderMissing,
    PlanSkippedRecent,
    PlanSkippedHidden,
    PlanSkippedTemp,
    PlanSkippedNoCategory,
    PlanSkippedExists,
    PlanSkippedOutside,
    PlanSkippedLocked,
    SameFolder,
    ClipboardInfo,
    FilesCount,
    OfTotal,
    WatchOn,
    WatchOff,
    StartWatch,
    StopWatch,
    WatchHint,
    DataFolder,
    ResetSettings,
    ResetTitle,
    ResetBody,
    ResetDone,
    RuleOrderHint,
    NoRules,
    NoRootsTitle,
    AddFolderHint,
    UndoConfirmTitle,
    UndoConfirmBody,
    Undone,
    TrashNote,
    SkippedNote,
    OpenFolder,
    CategoriesTitle,
    ExtensionHelp,
    LastRun,
    BackgroundLabel,
    BackgroundHint,
    AutostartLabel,
    AutostartHint,
    WatchRestartHint,
    TrayOpen,
    TrayToggle,
    TrayOrganize,
    TrayQuit,
    HomeReady,
    CategoryStats,
    SilentHint,
    DataTitle,
    Trash,
    EmptyPlanBody,
    SizeColumn,
    ExcludeHint,
    BuiltWith,
    LanguageSpanish,
    KindRegex,
    RuleNameRequired,
    RuleValueRequired,
    LanguageEnglish,
    TextSizeLabel,
    TextSmall,
    TextNormal,
    TextLarge,
    DensityLabel,
    DensityCompact,
    DensityComfortable,
    AccentCustom,
    AccentHexHint,
    AccentInvalid,
    AccentFillWarn,
    AccentBlue,
    AccentGreen,
    AccentPurple,
    AccentAmber,
    ApplyColor,
    ProfileTitle,
    ProfileGeneral,
    ProfileStudent,
    ProfileDeveloper,
    ProfilePhotographer,
    ProfileApply,
    ProfilePreviewTitle,
    ProfileApplied,
    DateSourceLabel,
    DateModified,
    DateCreated,
    DateToday,
    ExtDuplicateWarn,
    RegexUnavailable,
    RegexUnavailableCondition,
    RegexInvalid,
    CategoryNew,
    CategoryNameLabel,
    CategoryFolderLabel,
    CategoryColorLabel,
    CategoryColorNone,
    CategoryDeleteTitle,
    CategoryDeleteBody,
    CategoryFallbackLocked,
    CategoryFallbackHint,
    FolderProblemEmpty,
    FolderProblemInvalid,
    FolderProblemReserved,
    FolderProblemTrailing,
    FolderProblemTooLong,
    FolderProblemDuplicate,
    FolderRenameTitle,
    FolderRenameBody,
    FolderMoveContent,
    FolderLeaveContent,
    FolderMoved,
    ConditionsTitle,
    ConditionAdd,
    ConditionAll,
    ConditionAny,
    ConditionKind,
    ConditionValue,
    ConditionDays,
    ConditionUnit,
    ConditionRealImage,
    ConditionRealVideo,
    ConditionRealAudio,
    ConditionRealDocument,
    ConditionRealArchive,
    ConditionRealText,
    ConditionRealFont,
    ConditionRealProgram,
    ConditionRealBook,
    KindNameContains,
    KindNameStarts,
    KindNameEnds,
    KindMaxSize,
    KindNewer,
    KindCreatedOlder,
    KindCreatedNewer,
    KindRealType,
    KindSourceFolder,
    ActionCopy,
    ActionMoveRename,
    ActionCopyRename,
    ActionIgnore,
    RuleSubfolderLabel,
    RuleSubfolderHint,
    RuleTemplate,
    RuleTemplateHint,
    RuleSameFolderHint,
    RuleEdit,
    RuleSave,
    RuleUpdate,
    RuleCancel,
    RuleConditionsEmpty,
    RuleTester,
    RuleTesterName,
    RuleTesterSize,
    RuleTesterDays,
    RuleTesterMatch,
    RuleTesterNoMatch,
    RuleTesterLeave,
    RuleTesterRealTypeNote,
    RootOverrides,
    RootOwnSubfolders,
    RootExcludeFiles,
    RootExcludeDirs,
    RootOwnRules,
    RootOwnHint,
    RootEditRules,
    ScopeLabel,
    ScopeGlobal,
    TempExtensionsLabel,
    SkipDirsLabel,
    SystemFoldersNote,
    ExportConfig,
    ImportConfig,
    ImportDone,
    ImportInvalid,
    ImportPreviewTitle,
    ImportPreviewBody,
    ConfigExported,
    FolderAlreadyAdded,
    CopiedNotice,
    UndoCopyNote,
    Apply,
    RuleConditionRequired,
    RuleEnabled,
    CategoryEnabled,
    RootEnabled,
    DataFolderTitle,
    NoData,
    ExclusionsTitle,
    SourceFolderHint,
    ModeTitle,
    ModeSimple,
    ModeAdvanced,
    ModeHint,
    SectionStyle,
    StyleTitle,
    StyleHint,
    StyleByType,
    StyleByTypeAndDate,
    ShortcutsHint,
    FolderDownloads,
    FolderDesktop,
    FolderDocuments,
    FolderActive,
    FolderPaused,
    FolderAdded,
    FolderRemoved,
    FolderEmptyShortcuts,
    GroupTitle,
    GroupHint,
    GroupCount,
    GroupCountOne,
    GroupActive,
    GroupActiveHint,
    GroupPausedHint,
    GroupName,
    GroupTypes,
    GroupTypesEmpty,
    GroupAddType,
    GroupTypeHint,
    GroupColor,
    GroupColorAccent,
    GroupFolder,
    GroupFolderHint,
    GroupAdvanced,
    GroupRestore,
    GroupRestoreDone,
    GroupDelete,
    GroupDeleted,
    GroupAdd,
    GroupNoDelete,
    GroupTypeInUse,
    GroupNewName,
    RuleExampleName,
    RuleExampleSize,
    RuleExampleDays,
    RuleSpecialTitle,
    RuleListTitle,
    RuleSpecialHint,
    RuleWhen,
    RuleThen,
    RuleNameContains,
    RuleTypeIs,
    RuleWeighsMore,
    RuleOlderThan,
    RuleDestination,
    RuleMoreOptions,
    LivePreviewTitle,
    LivePreviewHint,
    LivePreviewGoes,
    LivePreviewStays,
    LivePreviewTrash,
    TooltipWatch,
    TooltipScope,
    TooltipSubfolders,
    TooltipSpecialRule,
    OnboardingTitle,
    OnboardingStep,
    OnboardingStep1Title,
    OnboardingStep1Body,
    OnboardingStep2Title,
    OnboardingStep2Body,
    OnboardingStep3Title,
    OnboardingStep3Body,
    OnboardingPick,
    OnboardingNoFolders,
    OnboardingFolderAdded,
    OnboardingPreview,
    OnboardingFinish,
    OnboardingSkip,
    Saved,
    UndoDisabledHint,
    OrganizeDisabledHint,
    TypePdf,
    TypeWord,
    TypeExcel,
    TypePowerPoint,
    TypeImage,
    TypeVideo,
    TypeAudio,
    TypeText,
    TypeArchive,
    TypeProgram,
    TypeBook,
    TypeFont,
    TypeOther,
    EmptyHomeTitle,
    EmptyHomeBody,
    EmptyPreviewBody,
    SaveListEmptyHint,
    AutoOrganize,
    AutoOrganizeHint,
    ProfileDescriptionBasic,
    ProfileDescriptionStudent,
    ProfileDescriptionDeveloper,
    ProfileDescriptionPhotographer,
    ProfileOnlyMissing,
    ProfileNothingNew,
    ApplyChanges,
    DropHere,
    DuplicatesTitle,
    DuplicatesIntro,
    DuplicatesIdentical,
    FindDuplicates,
    DuplicatesScanning,
    DuplicatesCancel,
    DuplicatesEmpty,
    DuplicatesKeep,
    DuplicatesDelete,
    DuplicatesFreed,
    DuplicatesAction,
    DuplicatesNoSelection,
    ProfileSkipped,
    ViewChanges,
    StepChooseFolder,
    StepReviewFiles,
    StepMoveFiles,
    RuleConditionValueRequired,
    RuleConditionNumber,
    RuleDestinationRequired,
    HistoryReviewHint,
    RestartWatch,
    WatchFailed,
    WatchFailedShort,
    WatchRetrying,
    GuiStartFailed,
    SwitchOn,
    SwitchOff,
    StateExpanded,
    StateCollapsed,
}

/// Todas las claves, para el test que comprueba que ninguna quede vacia.
pub const ALL_KEYS: &[Key] = &[
    Key::AppName,
    Key::TabHome,
    Key::TabPreview,
    Key::TabOrganize,
    Key::TabHistory,
    Key::TabSettings,
    Key::OrganizeNow,
    Key::Undo,
    Key::Rescan,
    Key::ChooseFolder,
    Key::AddFolder,
    Key::Remove,
    Key::WizardStep,
    Key::WizardTitle,
    Key::WizardBody,
    Key::WizardNext,
    Key::WizardBack,
    Key::WizardLast,
    Key::Watching,
    Key::LastOrganizeNever,
    Key::LastOrganize,
    Key::Scanning,
    Key::RecentActivity,
    Key::SeeAll,
    Key::ColTime,
    Key::ColAction,
    Key::ColFiles,
    Key::ColDestination,
    Key::TotalOrganized,
    Key::SpaceFreed,
    Key::ActiveRules,
    Key::PreviewTitle,
    Key::PreviewSummary,
    Key::PreviewEmpty,
    Key::PreviewSkipped,
    Key::PreviewFilter,
    Key::PreviewSelectAll,
    Key::MovedNotice,
    Key::Category,
    Key::Extensions,
    Key::AddExtension,
    Key::ExtensionPlaceholder,
    Key::SubfoldersTitle,
    Key::SubfoldersLabel,
    Key::SubByDate,
    Key::SubByDateHint,
    Key::SubByType,
    Key::SubByTypeHint,
    Key::SubByCapture,
    Key::SubByCaptureHint,
    Key::GeneralTitle,
    Key::FoldersTitle,
    Key::RulesTitle,
    Key::WatchTitle,
    Key::AppearanceTitle,
    Key::AdvancedTitle,
    Key::ThemeLabel,
    Key::ThemeDark,
    Key::ThemeLight,
    Key::ThemeSystem,
    Key::LanguageLabel,
    Key::SilentMode,
    Key::NotificationsTitle,
    Key::SilentOn,
    Key::SilentOff,
    Key::AdvancedMode,
    Key::AdvancedHint,
    Key::SilenceBar,
    Key::ScheduleLabel,
    Key::ScheduleOff,
    Key::DebounceLabel,
    Key::RecentGuardLabel,
    Key::ExcludeLabel,
    Key::AccentLabel,
    Key::VersionLabel,
    Key::CustomRules,
    Key::RuleUp,
    Key::RuleDown,
    Key::RuleDelete,
    Key::RuleAdd,
    Key::RuleName,
    Key::RuleKind,
    Key::RuleValue,
    Key::RuleAction,
    Key::RuleTarget,
    Key::KindExtension,
    Key::KindGlob,
    Key::KindMinSize,
    Key::KindOlder,
    Key::ActionMove,
    Key::ActionTrash,
    Key::ActionRename,
    Key::ActionRenameUnavailable,
    Key::HistoryTitle,
    Key::HistoryEmptyTitle,
    Key::HistoryEmpty,
    Key::HistoryUndo,
    Key::ExportCsv,
    Key::ConfirmTrashTitle,
    Key::ConfirmTrashBody,
    Key::ConfirmMoveTitle,
    Key::ConfirmMoveBody,
    Key::Cancel,
    Key::ConfirmTrashYes,
    Key::ConfirmMoveYes,
    Key::EmptyNoRoots,
    Key::EmptyNoPlan,
    Key::ErrorTitle,
    Key::Dismiss,
    Key::BatchLabel,
    Key::FilesLabel,
    Key::ItemsLabel,
    Key::UndoDone,
    Key::UndoNothing,
    Key::UndoPartial,
    Key::NothingToUndo,
    Key::ExportDone,
    Key::ExportEmpty,
    Key::ScheduleMinutes,
    Key::DaysLabel,
    Key::SizeLabel,
    Key::BytesLabel,
    Key::FolderMissing,
    Key::PlanSkippedRecent,
    Key::PlanSkippedHidden,
    Key::PlanSkippedTemp,
    Key::PlanSkippedNoCategory,
    Key::PlanSkippedExists,
    Key::PlanSkippedOutside,
    Key::PlanSkippedLocked,
    Key::SameFolder,
    Key::ClipboardInfo,
    Key::FilesCount,
    Key::OfTotal,
    Key::WatchOn,
    Key::WatchOff,
    Key::StartWatch,
    Key::StopWatch,
    Key::WatchHint,
    Key::DataFolder,
    Key::ResetSettings,
    Key::ResetTitle,
    Key::ResetBody,
    Key::ResetDone,
    Key::RuleOrderHint,
    Key::NoRules,
    Key::NoRootsTitle,
    Key::AddFolderHint,
    Key::UndoConfirmTitle,
    Key::UndoConfirmBody,
    Key::Undone,
    Key::TrashNote,
    Key::SkippedNote,
    Key::OpenFolder,
    Key::CategoriesTitle,
    Key::ExtensionHelp,
    Key::LastRun,
    Key::BackgroundLabel,
    Key::BackgroundHint,
    Key::AutostartLabel,
    Key::AutostartHint,
    Key::WatchRestartHint,
    Key::TrayOpen,
    Key::TrayToggle,
    Key::TrayOrganize,
    Key::TrayQuit,
    Key::HomeReady,
    Key::CategoryStats,
    Key::SilentHint,
    Key::DataTitle,
    Key::Trash,
    Key::EmptyPlanBody,
    Key::SizeColumn,
    Key::ExcludeHint,
    Key::BuiltWith,
    Key::LanguageSpanish,
    Key::KindRegex,
    Key::RuleNameRequired,
    Key::RuleValueRequired,
    Key::LanguageEnglish,
    Key::TextSizeLabel,
    Key::TextSmall,
    Key::TextNormal,
    Key::TextLarge,
    Key::DensityLabel,
    Key::DensityCompact,
    Key::DensityComfortable,
    Key::AccentCustom,
    Key::AccentHexHint,
    Key::AccentInvalid,
    Key::AccentFillWarn,
    Key::AccentBlue,
    Key::AccentGreen,
    Key::AccentPurple,
    Key::AccentAmber,
    Key::ApplyColor,
    Key::ProfileTitle,
    Key::ProfileGeneral,
    Key::ProfileStudent,
    Key::ProfileDeveloper,
    Key::ProfilePhotographer,
    Key::ProfileApply,
    Key::ProfilePreviewTitle,
    Key::ProfileApplied,
    Key::DateSourceLabel,
    Key::DateModified,
    Key::DateCreated,
    Key::DateToday,
    Key::ExtDuplicateWarn,
    Key::RegexUnavailable,
    Key::RegexUnavailableCondition,
    Key::RegexInvalid,
    Key::CategoryNew,
    Key::CategoryNameLabel,
    Key::CategoryFolderLabel,
    Key::CategoryColorLabel,
    Key::CategoryColorNone,
    Key::CategoryDeleteTitle,
    Key::CategoryDeleteBody,
    Key::CategoryFallbackLocked,
    Key::CategoryFallbackHint,
    Key::FolderProblemEmpty,
    Key::FolderProblemInvalid,
    Key::FolderProblemReserved,
    Key::FolderProblemTrailing,
    Key::FolderProblemTooLong,
    Key::FolderProblemDuplicate,
    Key::FolderRenameTitle,
    Key::FolderRenameBody,
    Key::FolderMoveContent,
    Key::FolderLeaveContent,
    Key::FolderMoved,
    Key::ConditionsTitle,
    Key::ConditionAdd,
    Key::ConditionAll,
    Key::ConditionAny,
    Key::ConditionKind,
    Key::ConditionValue,
    Key::ConditionDays,
    Key::ConditionUnit,
    Key::ConditionRealImage,
    Key::ConditionRealVideo,
    Key::ConditionRealAudio,
    Key::ConditionRealDocument,
    Key::ConditionRealArchive,
    Key::ConditionRealText,
    Key::ConditionRealFont,
    Key::ConditionRealProgram,
    Key::ConditionRealBook,
    Key::KindNameContains,
    Key::KindNameStarts,
    Key::KindNameEnds,
    Key::KindMaxSize,
    Key::KindNewer,
    Key::KindCreatedOlder,
    Key::KindCreatedNewer,
    Key::KindRealType,
    Key::KindSourceFolder,
    Key::ActionCopy,
    Key::ActionMoveRename,
    Key::ActionCopyRename,
    Key::ActionIgnore,
    Key::RuleSubfolderLabel,
    Key::RuleSubfolderHint,
    Key::RuleTemplate,
    Key::RuleTemplateHint,
    Key::RuleSameFolderHint,
    Key::RuleEdit,
    Key::RuleSave,
    Key::RuleUpdate,
    Key::RuleCancel,
    Key::RuleConditionsEmpty,
    Key::RuleTester,
    Key::RuleTesterName,
    Key::RuleTesterSize,
    Key::RuleTesterDays,
    Key::RuleTesterMatch,
    Key::RuleTesterNoMatch,
    Key::RuleTesterLeave,
    Key::RuleTesterRealTypeNote,
    Key::RootOverrides,
    Key::RootOwnSubfolders,
    Key::RootExcludeFiles,
    Key::RootExcludeDirs,
    Key::RootOwnRules,
    Key::RootOwnHint,
    Key::RootEditRules,
    Key::ScopeLabel,
    Key::ScopeGlobal,
    Key::TempExtensionsLabel,
    Key::SkipDirsLabel,
    Key::SystemFoldersNote,
    Key::ExportConfig,
    Key::ImportConfig,
    Key::ImportDone,
    Key::ImportInvalid,
    Key::ImportPreviewTitle,
    Key::ImportPreviewBody,
    Key::ConfigExported,
    Key::FolderAlreadyAdded,
    Key::CopiedNotice,
    Key::UndoCopyNote,
    Key::Apply,
    Key::RuleConditionRequired,
    Key::RuleEnabled,
    Key::CategoryEnabled,
    Key::RootEnabled,
    Key::DataFolderTitle,
    Key::NoData,
    Key::ExclusionsTitle,
    Key::SourceFolderHint,
    Key::ModeTitle,
    Key::ModeSimple,
    Key::ModeAdvanced,
    Key::ModeHint,
    Key::SectionStyle,
    Key::StyleTitle,
    Key::StyleHint,
    Key::StyleByType,
    Key::StyleByTypeAndDate,
    Key::ShortcutsHint,
    Key::FolderDownloads,
    Key::FolderDesktop,
    Key::FolderDocuments,
    Key::FolderActive,
    Key::FolderPaused,
    Key::FolderAdded,
    Key::FolderRemoved,
    Key::FolderEmptyShortcuts,
    Key::GroupTitle,
    Key::GroupHint,
    Key::GroupCount,
    Key::GroupCountOne,
    Key::GroupActive,
    Key::GroupActiveHint,
    Key::GroupPausedHint,
    Key::GroupName,
    Key::GroupTypes,
    Key::GroupTypesEmpty,
    Key::GroupAddType,
    Key::GroupTypeHint,
    Key::GroupColor,
    Key::GroupColorAccent,
    Key::GroupFolder,
    Key::GroupFolderHint,
    Key::GroupAdvanced,
    Key::GroupRestore,
    Key::GroupRestoreDone,
    Key::GroupDelete,
    Key::GroupDeleted,
    Key::GroupAdd,
    Key::GroupNoDelete,
    Key::GroupTypeInUse,
    Key::GroupNewName,
    Key::RuleExampleName,
    Key::RuleExampleSize,
    Key::RuleExampleDays,
    Key::RuleSpecialTitle,
    Key::RuleListTitle,
    Key::RuleSpecialHint,
    Key::RuleWhen,
    Key::RuleThen,
    Key::RuleNameContains,
    Key::RuleTypeIs,
    Key::RuleWeighsMore,
    Key::RuleOlderThan,
    Key::RuleDestination,
    Key::RuleMoreOptions,
    Key::LivePreviewTitle,
    Key::LivePreviewHint,
    Key::LivePreviewGoes,
    Key::LivePreviewStays,
    Key::LivePreviewTrash,
    Key::TooltipWatch,
    Key::TooltipScope,
    Key::TooltipSubfolders,
    Key::TooltipSpecialRule,
    Key::OnboardingTitle,
    Key::OnboardingStep,
    Key::OnboardingStep1Title,
    Key::OnboardingStep1Body,
    Key::OnboardingStep2Title,
    Key::OnboardingStep2Body,
    Key::OnboardingStep3Title,
    Key::OnboardingStep3Body,
    Key::OnboardingPick,
    Key::OnboardingNoFolders,
    Key::OnboardingFolderAdded,
    Key::OnboardingPreview,
    Key::OnboardingFinish,
    Key::OnboardingSkip,
    Key::Saved,
    Key::UndoDisabledHint,
    Key::OrganizeDisabledHint,
    Key::TypePdf,
    Key::TypeWord,
    Key::TypeExcel,
    Key::TypePowerPoint,
    Key::TypeImage,
    Key::TypeVideo,
    Key::TypeAudio,
    Key::TypeText,
    Key::TypeArchive,
    Key::TypeProgram,
    Key::TypeBook,
    Key::TypeFont,
    Key::TypeOther,
    Key::EmptyHomeTitle,
    Key::EmptyHomeBody,
    Key::EmptyPreviewBody,
    Key::SaveListEmptyHint,
    Key::AutoOrganize,
    Key::AutoOrganizeHint,
    Key::ProfileDescriptionBasic,
    Key::ProfileDescriptionStudent,
    Key::ProfileDescriptionDeveloper,
    Key::ProfileDescriptionPhotographer,
    Key::ProfileOnlyMissing,
    Key::ProfileNothingNew,
    Key::ApplyChanges,
    Key::DropHere,
    Key::DuplicatesTitle,
    Key::DuplicatesIntro,
    Key::DuplicatesIdentical,
    Key::FindDuplicates,
    Key::DuplicatesScanning,
    Key::DuplicatesCancel,
    Key::DuplicatesEmpty,
    Key::DuplicatesKeep,
    Key::DuplicatesDelete,
    Key::DuplicatesFreed,
    Key::DuplicatesAction,
    Key::DuplicatesNoSelection,
    Key::ProfileSkipped,
    Key::ViewChanges,
    Key::StepChooseFolder,
    Key::StepReviewFiles,
    Key::StepMoveFiles,
    Key::RuleConditionValueRequired,
    Key::RuleConditionNumber,
    Key::RuleDestinationRequired,
    Key::HistoryReviewHint,
    Key::RestartWatch,
    Key::WatchFailed,
    Key::WatchFailedShort,
    Key::WatchRetrying,
    Key::GuiStartFailed,
    Key::SwitchOn,
    Key::SwitchOff,
    Key::StateExpanded,
    Key::StateCollapsed,
];

pub fn t(lang: Lang, key: Key) -> &'static str {
    use Key as K;
    match (lang, key) {
        (_, K::AppName) => "Smart Organizer",
        (Lang::Es, K::TabHome) => "Inicio",
        (Lang::En, K::TabHome) => "Home",
        (Lang::Es, K::TabPreview) => "Vista previa",
        (Lang::En, K::TabPreview) => "Preview",
        (Lang::Es, K::TabOrganize) => "Organizar",
        (Lang::En, K::TabOrganize) => "Organize",
        (Lang::Es, K::TabHistory) => "Historial",
        (Lang::En, K::TabHistory) => "History",
        (Lang::Es, K::TabSettings) => "Ajustes",
        (Lang::En, K::TabSettings) => "Settings",
        (Lang::Es, K::OrganizeNow) => "Organizar ahora",
        (Lang::En, K::OrganizeNow) => "Organize now",
        (Lang::Es, K::Undo) => "Deshacer",
        (Lang::En, K::Undo) => "Undo",
        (Lang::Es, K::Rescan) => "Volver a escanear",
        (Lang::En, K::Rescan) => "Scan again",
        (Lang::Es, K::ChooseFolder) => "Elegir carpeta",
        (Lang::En, K::ChooseFolder) => "Choose folder",
        (Lang::Es, K::AddFolder) => "Añadir carpeta",
        (Lang::En, K::AddFolder) => "Add folder",
        (Lang::Es, K::Remove) => "Quitar",
        (Lang::En, K::Remove) => "Remove",
        (Lang::Es, K::WizardStep) => "Paso",
        (Lang::En, K::WizardStep) => "Step",
        (Lang::Es, K::WizardTitle) => "Elige la carpeta donde caen tus archivos",
        (Lang::En, K::WizardTitle) => "Choose the folder where your files land",
        (Lang::Es, K::WizardBody) => {
            "Lo normal es el Escritorio o Descargas. Nada se mueve hasta que lo confirmes."
        }
        (Lang::En, K::WizardBody) => {
            "Usually Desktop or Downloads. Nothing moves until you confirm."
        }
        (Lang::Es, K::WizardNext) => "Ver que se movería",
        (Lang::En, K::WizardNext) => "See what would move",
        (Lang::Es, K::WizardBack) => "Volver",
        (Lang::En, K::WizardBack) => "Back",
        (Lang::Es, K::WizardLast) => "Mover los archivos",
        (Lang::En, K::WizardLast) => "Move the files",
        (Lang::Es, K::Watching) => "Vigilando",
        (Lang::En, K::Watching) => "Watching",
        (Lang::Es, K::LastOrganizeNever) => "Todavía no se organizó nada",
        (Lang::En, K::LastOrganizeNever) => "Nothing organized yet",
        (Lang::Es, K::LastOrganize) => "Última organización",
        (Lang::En, K::LastOrganize) => "Last organized",
        (Lang::Es, K::Scanning) => "Escaneando",
        (Lang::En, K::Scanning) => "Scanning",
        (Lang::Es, K::RecentActivity) => "Actividad reciente",
        (Lang::En, K::RecentActivity) => "Recent activity",
        (Lang::Es, K::SeeAll) => "Ver historial completo",
        (Lang::En, K::SeeAll) => "See full history",
        (Lang::Es, K::ColTime) => "Hora",
        (Lang::En, K::ColTime) => "Time",
        (Lang::Es, K::ColAction) => "Acción",
        (Lang::En, K::ColAction) => "Action",
        (Lang::Es, K::ColFiles) => "Archivos",
        (Lang::En, K::ColFiles) => "Files",
        (Lang::Es, K::ColDestination) => "Destino",
        (Lang::En, K::ColDestination) => "Destination",
        (Lang::Es, K::TotalOrganized) => "Archivos organizados",
        (Lang::En, K::TotalOrganized) => "Files organized",
        (Lang::Es, K::SpaceFreed) => "Espacio movido",
        (Lang::En, K::SpaceFreed) => "Space moved",
        (Lang::Es, K::ActiveRules) => "Reglas y grupos activos",
        (Lang::En, K::ActiveRules) => "Active rules and groups",
        (Lang::Es, K::PreviewTitle) => "Esto es lo que se movería",
        (Lang::En, K::PreviewTitle) => "This is what would move",
        (Lang::Es, K::PreviewSummary) => "Se moverían {files} archivos a {destinations}.",
        (Lang::En, K::PreviewSummary) => "{files} files would move to {destinations}.",
        (Lang::Es, K::PreviewEmpty) => "No hay archivos para mover en este momento.",
        (Lang::En, K::PreviewEmpty) => "There are no files to move right now.",
        (Lang::Es, K::PreviewSkipped) => "No se tocan",
        (Lang::En, K::PreviewSkipped) => "Left alone",
        (Lang::Es, K::PreviewFilter) => "Filtrar por nombre",
        (Lang::En, K::PreviewFilter) => "Filter by name",
        (Lang::Es, K::PreviewSelectAll) => "Marcar todos",
        (Lang::En, K::PreviewSelectAll) => "Select all",
        (Lang::Es, K::MovedNotice) => {
            "Se movieron {files} archivos. Puedes deshacerlo desde el historial."
        }
        (Lang::En, K::MovedNotice) => "Moved {files} files. You can undo it from history.",
        (Lang::Es, K::Category) => "Grupo",
        (Lang::En, K::Category) => "Group",
        (Lang::Es, K::Extensions) => "Tipos de archivo",
        (Lang::En, K::Extensions) => "File types",
        (Lang::Es, K::AddExtension) => "Añadir",
        (Lang::En, K::AddExtension) => "Add",
        (Lang::Es, K::ExtensionPlaceholder) => "Ej: .psd",
        (Lang::En, K::ExtensionPlaceholder) => "E.g. .psd",
        (Lang::Es, K::SubfoldersTitle) => "Organización automática dentro de las carpetas",
        (Lang::En, K::SubfoldersTitle) => "Automatic organization inside folders",
        (Lang::Es, K::SubfoldersLabel) => "Subcarpetas",
        (Lang::En, K::SubfoldersLabel) => "Subfolders",
        (Lang::Es, K::SubByDate) => "Crear subcarpetas por año y mes",
        (Lang::En, K::SubByDate) => "Create year and month subfolders",
        (Lang::Es, K::SubByDateHint) => "Ej: Imágenes/2026/10/",
        (Lang::En, K::SubByDateHint) => "E.g. Images/2026/10/",
        (Lang::Es, K::SubByType) => "Crear subcarpetas por tipo de archivo",
        (Lang::En, K::SubByType) => "Create subfolders by file type",
        (Lang::Es, K::SubByTypeHint) => "Ej: Imágenes/PNG/",
        (Lang::En, K::SubByTypeHint) => "E.g. Images/PNG/",
        (Lang::Es, K::SubByCapture) => "Separar capturas de pantalla",
        (Lang::En, K::SubByCapture) => "Separate screenshots",
        (Lang::Es, K::SubByCaptureHint) => "Ej: Imágenes/Capturas/",
        (Lang::En, K::SubByCaptureHint) => "E.g. Images/Screenshots/",
        (Lang::Es, K::GeneralTitle) => "General",
        (Lang::En, K::GeneralTitle) => "General",
        (Lang::Es, K::FoldersTitle) => "Carpetas",
        (Lang::En, K::FoldersTitle) => "Folders",
        (Lang::Es, K::RulesTitle) => "Reglas especiales",
        (Lang::En, K::RulesTitle) => "Special rules",
        (Lang::Es, K::WatchTitle) => "Vigilancia",
        (Lang::En, K::WatchTitle) => "Watching",
        (Lang::Es, K::AppearanceTitle) => "Apariencia",
        (Lang::En, K::AppearanceTitle) => "Appearance",
        (Lang::Es, K::AdvancedTitle) => "Avanzado",
        (Lang::En, K::AdvancedTitle) => "Advanced",
        (Lang::Es, K::ThemeLabel) => "Tema",
        (Lang::En, K::ThemeLabel) => "Theme",
        (Lang::Es, K::ThemeDark) => "Oscuro",
        (Lang::En, K::ThemeDark) => "Dark",
        (Lang::Es, K::ThemeLight) => "Claro",
        (Lang::En, K::ThemeLight) => "Light",
        (Lang::Es, K::ThemeSystem) => "Seguir al sistema",
        (Lang::En, K::ThemeSystem) => "Follow system",
        (Lang::Es, K::LanguageLabel) => "Idioma",
        (Lang::En, K::LanguageLabel) => "Language",
        (Lang::Es, K::SilentMode) => "Modo silencioso",
        (Lang::En, K::SilentMode) => "Silent mode",
        (Lang::Es, K::NotificationsTitle) => "Avisos",
        (Lang::En, K::NotificationsTitle) => "Notifications",
        (Lang::Es, K::SilentOn) => "Silencio: activado",
        (Lang::En, K::SilentOn) => "Silent: on",
        (Lang::Es, K::SilentOff) => "Silencio: desactivado",
        (Lang::En, K::SilentOff) => "Silent: off",
        (Lang::Es, K::AdvancedMode) => "Modo avanzado",
        (Lang::En, K::AdvancedMode) => "Advanced mode",
        (Lang::Es, K::AdvancedHint) => {
            "Muestra Organizar y las reglas especiales. El modo Sencillo deja Inicio, Vista previa, Historial y Ajustes."
        }
        (Lang::En, K::AdvancedHint) => {
            "Shows Organize and special rules. Simple mode keeps Home, Preview, History and Settings."
        }
        (Lang::Es, K::SilenceBar) => "Modo silencioso",
        (Lang::En, K::SilenceBar) => "Silent mode",
        (Lang::Es, K::ScheduleLabel) => "Organizar cada (minutos)",
        (Lang::En, K::ScheduleLabel) => "Organize every (minutes)",
        (Lang::Es, K::ScheduleOff) => "Solo cuando haya cambios",
        (Lang::En, K::ScheduleOff) => "Only when something changes",
        (Lang::Es, K::DebounceLabel) => "Esperar antes de actuar (milisegundos)",
        (Lang::En, K::DebounceLabel) => "Wait before acting (milliseconds)",
        (Lang::Es, K::RecentGuardLabel) => "No tocar archivos de los últimos (minutos)",
        (Lang::En, K::RecentGuardLabel) => "Leave files alone for the last (minutes)",
        (Lang::Es, K::ExcludeLabel) => "Carpetas excluidas",
        (Lang::En, K::ExcludeLabel) => "Excluded folders",
        (Lang::Es, K::AccentLabel) => "Color de acento",
        (Lang::En, K::AccentLabel) => "Accent color",
        (Lang::Es, K::VersionLabel) => "Versión",
        (Lang::En, K::VersionLabel) => "Version",
        (Lang::Es, K::CustomRules) => {
            "Reglas especiales por tipo de archivo, patrón, tamaño o antigüedad"
        }
        (Lang::En, K::CustomRules) => "Special rules by file type, pattern, size or age",
        (Lang::Es, K::RuleUp) => "Subir",
        (Lang::En, K::RuleUp) => "Up",
        (Lang::Es, K::RuleDown) => "Bajar",
        (Lang::En, K::RuleDown) => "Down",
        (Lang::Es, K::RuleDelete) => "Borrar",
        (Lang::En, K::RuleDelete) => "Delete",
        (Lang::Es, K::RuleAdd) => "Añadir regla especial",
        (Lang::En, K::RuleAdd) => "Add special rule",
        (Lang::Es, K::RuleName) => "Nombre",
        (Lang::En, K::RuleName) => "Name",
        (Lang::Es, K::RuleKind) => "Cuando",
        (Lang::En, K::RuleKind) => "When",
        (Lang::Es, K::RuleValue) => "Valor",
        (Lang::En, K::RuleValue) => "Value",
        (Lang::Es, K::RuleAction) => "Hacer",
        (Lang::En, K::RuleAction) => "Then",
        (Lang::Es, K::RuleTarget) => "Destino",
        (Lang::En, K::RuleTarget) => "Destination",
        (Lang::Es, K::KindExtension) => "El tipo de archivo es",
        (Lang::En, K::KindExtension) => "File type is",
        (Lang::Es, K::KindGlob) => "El nombre coincide con",
        (Lang::En, K::KindGlob) => "Name matches",
        (Lang::Es, K::KindMinSize) => "Pesa al menos (MB)",
        (Lang::En, K::KindMinSize) => "At least (MB)",
        (Lang::Es, K::KindOlder) => "Tiene más de (días)",
        (Lang::En, K::KindOlder) => "Older than (days)",
        (Lang::Es, K::ActionMove) => "Mover a",
        (Lang::En, K::ActionMove) => "Move to",
        (Lang::Es, K::ActionTrash) => "Mandar a la papelera",
        (Lang::En, K::ActionTrash) => "Send to trash",
        (Lang::Es, K::ActionRename) => "Renombrar",
        (Lang::En, K::ActionRename) => "Rename",
        (Lang::Es, K::ActionRenameUnavailable) => {
            "Renombrar en el mismo sitio puede volver a aplicarse sin fin. Usa Mover y renombrar, o Copiar y renombrar."
        }
        (Lang::En, K::ActionRenameUnavailable) => {
            "Renaming in place can apply again forever. Use Move and rename, or Copy and rename."
        }
        (Lang::Es, K::HistoryTitle) => "Historial",
        (Lang::En, K::HistoryTitle) => "History",
        (Lang::Es, K::HistoryEmptyTitle) => "Sin actividad",
        (Lang::En, K::HistoryEmptyTitle) => "No activity yet",
        (Lang::Es, K::HistoryEmpty) => {
            "Todavía no hay nada en el historial. Cuando organices archivos, aparecerán aquí y podrás deshacer."
        }
        (Lang::En, K::HistoryEmpty) => {
            "History is empty. Once you organize files they appear here and can be undone."
        }
        (Lang::Es, K::HistoryUndo) => "Deshacer este lote",
        (Lang::En, K::HistoryUndo) => "Undo this batch",
        (Lang::Es, K::ExportCsv) => "Guardar la lista",
        (Lang::En, K::ExportCsv) => "Save the list",
        (Lang::Es, K::ConfirmTrashTitle) => "Mover a la papelera",
        (Lang::En, K::ConfirmTrashTitle) => "Move to trash",
        (Lang::Es, K::ConfirmTrashBody) => {
            "Se moverían {files} archivos a la Papelera del sistema. Podrás recuperarlos desde la Papelera y deshacer queda registrado en el historial."
        }
        (Lang::En, K::ConfirmTrashBody) => {
            "{files} files would move to the system Trash. You can restore them from the Trash and the action is recorded in history."
        }
        (Lang::Es, K::ConfirmMoveTitle) => "Mover archivos",
        (Lang::En, K::ConfirmMoveTitle) => "Move files",
        (Lang::Es, K::ConfirmMoveBody) => {
            "Se moverían {files} archivos. Nada se borra: los originales se conservan y el lote se puede deshacer."
        }
        (Lang::En, K::ConfirmMoveBody) => {
            "{files} files would move. Nothing is deleted: originals are kept and the batch can be undone."
        }
        (Lang::Es, K::Cancel) => "Cancelar",
        (Lang::En, K::Cancel) => "Cancel",
        (Lang::Es, K::ConfirmTrashYes) => "Sí, mover a la papelera",
        (Lang::En, K::ConfirmTrashYes) => "Yes, move to trash",
        (Lang::Es, K::ConfirmMoveYes) => "Sí, mover",
        (Lang::En, K::ConfirmMoveYes) => "Yes, move",
        (Lang::Es, K::EmptyNoRoots) => "Elige una carpeta para empezar. Por ejemplo el Escritorio.",
        (Lang::En, K::EmptyNoRoots) => "Choose a folder to start. Desktop is a common choice.",
        (Lang::Es, K::EmptyNoPlan) => "Está todo ordenado. No hay nada pendiente.",
        (Lang::En, K::EmptyNoPlan) => "Everything is in place. Nothing pending.",
        (Lang::Es, K::ErrorTitle) => "No se pudo completar",
        (Lang::En, K::ErrorTitle) => "Could not complete",
        (Lang::Es, K::Dismiss) => "Entendido",
        (Lang::En, K::Dismiss) => "Got it",
        (Lang::Es, K::BatchLabel) => "Lote",
        (Lang::En, K::BatchLabel) => "Batch",
        (Lang::Es, K::FilesLabel) => "archivos",
        (Lang::En, K::FilesLabel) => "files",
        (Lang::Es, K::ItemsLabel) => "Elementos",
        (Lang::En, K::ItemsLabel) => "Items",
        (Lang::Es, K::UndoDone) => "Se devolvieron {files} archivos a su sitio.",
        (Lang::En, K::UndoDone) => "Returned {files} files to their place.",
        (Lang::Es, K::UndoNothing) => "No se pudo deshacer nada de este lote.",
        (Lang::En, K::UndoNothing) => "Nothing in this batch could be undone.",
        (Lang::Es, K::UndoPartial) => {
            "Se devolvieron {files} archivos. {skipped} no se pudieron mover porque su sitio original está ocupado o el archivo ya no existe."
        }
        (Lang::En, K::UndoPartial) => {
            "Returned {files} files. {skipped} could not move because their original place is taken or the file is gone."
        }
        (Lang::Es, K::NothingToUndo) => "No hay ningún lote para deshacer.",
        (Lang::En, K::NothingToUndo) => "There is no batch to undo.",
        (Lang::Es, K::ExportDone) => "Historial exportado a {path}",
        (Lang::En, K::ExportDone) => "History exported to {path}",
        (Lang::Es, K::ExportEmpty) => "No hay historial para exportar.",
        (Lang::En, K::ExportEmpty) => "There is no history to export.",
        (Lang::Es, K::ScheduleMinutes) => "minutos",
        (Lang::En, K::ScheduleMinutes) => "minutes",
        (Lang::Es, K::DaysLabel) => "días",
        (Lang::En, K::DaysLabel) => "days",
        (Lang::Es, K::SizeLabel) => "MB",
        (Lang::En, K::SizeLabel) => "MB",
        (Lang::Es, K::BytesLabel) => "B",
        (Lang::En, K::BytesLabel) => "B",
        (Lang::Es, K::FolderMissing) => "La carpeta ya no existe",
        (Lang::En, K::FolderMissing) => "Folder no longer exists",
        (Lang::Es, K::PlanSkippedRecent) => "es demasiado reciente",
        (Lang::En, K::PlanSkippedRecent) => "is too recent",
        (Lang::Es, K::PlanSkippedHidden) => "está oculto",
        (Lang::En, K::PlanSkippedHidden) => "is hidden",
        (Lang::Es, K::PlanSkippedTemp) => "parece una descarga a medías",
        (Lang::En, K::PlanSkippedTemp) => "looks like a partial download",
        (Lang::Es, K::PlanSkippedNoCategory) => "no tiene grupo",
        (Lang::En, K::PlanSkippedNoCategory) => "has no group",
        (Lang::Es, K::PlanSkippedExists) => "ya está en su sitio",
        (Lang::En, K::PlanSkippedExists) => "is already in place",
        (Lang::Es, K::PlanSkippedOutside) => "queda fuera de la carpeta elegida",
        (Lang::En, K::PlanSkippedOutside) => "is outside the chosen folder",
        (Lang::Es, K::PlanSkippedLocked) => "está en uso por otro programa",
        (Lang::En, K::PlanSkippedLocked) => "is in use by another program",
        (Lang::Es, K::SameFolder) => "elegida",
        (Lang::En, K::SameFolder) => "chosen",
        (Lang::Es, K::ClipboardInfo) => {
            "Ctrl+Z deshace, Ctrl+O abre una carpeta, F5 vuelve a escanear."
        }
        (Lang::En, K::ClipboardInfo) => "Ctrl+Z undoes, Ctrl+O opens a folder, F5 scans again.",
        (Lang::Es, K::FilesCount) => "{files} archivos",
        (Lang::En, K::FilesCount) => "{files} files",
        (Lang::Es, K::OfTotal) => "de",
        (Lang::En, K::OfTotal) => "of",
        (Lang::Es, K::WatchOn) => "Vigilancia activa",
        (Lang::En, K::WatchOn) => "Watching",
        (Lang::Es, K::WatchOff) => "Vigilancia apagada",
        (Lang::En, K::WatchOff) => "Not watching",
        (Lang::Es, K::StartWatch) => "Activar vigilancia",
        (Lang::En, K::StartWatch) => "Start watching",
        (Lang::Es, K::StopWatch) => "Detener vigilancia",
        (Lang::En, K::StopWatch) => "Stop watching",
        (Lang::Es, K::WatchHint) => {
            "Mientras la vigilancia está activa, los archivos nuevos se organizan solos. Corre en un proceso aparte; si cierras la ventana, sigue en la bandeja."
        }
        (Lang::En, K::WatchHint) => {
            "While watching is on, new files are organized automatically. It runs in a separate process; closing the window leaves it in the tray."
        }
        (Lang::Es, K::DataFolder) => "Abrir carpeta de datos",
        (Lang::En, K::DataFolder) => "Open data folder",
        (Lang::Es, K::ResetSettings) => "Restablecer ajustes",
        (Lang::En, K::ResetSettings) => "Reset settings",
        (Lang::Es, K::ResetTitle) => "Restablecer ajustes",
        (Lang::En, K::ResetTitle) => "Reset settings",
        (Lang::Es, K::ResetBody) => {
            "Se perderían tus carpetas, reglas y preferencias. El historial y tus archivos no se tocan."
        }
        (Lang::En, K::ResetBody) => {
            "Your folders, rules and preferences would be lost. History and your files are not touched."
        }
        (Lang::Es, K::ResetDone) => "Ajustes restablecidos.",
        (Lang::En, K::ResetDone) => "Settings reset.",
        (Lang::Es, K::RuleOrderHint) => {
            "Un archivo factura.pdf irá a Documentos. Las reglas especiales se aplican antes que los grupos."
        }
        (Lang::En, K::RuleOrderHint) => {
            "A factura.pdf file will go to Documents. Special rules run before groups."
        }
        (Lang::Es, K::NoRules) => {
            "No hay reglas especiales. Los grupos por tipo de archivo siguen funcionando."
        }
        (Lang::En, K::NoRules) => "No special rules. File-type groups still work.",
        (Lang::Es, K::NoRootsTitle) => "Todavía no hay carpetas",
        (Lang::En, K::NoRootsTitle) => "No folders yet",
        (Lang::Es, K::AddFolderHint) => {
            "Añade el Escritorio, Descargas o la carpeta donde guardes tus archivos; nada se mueve hasta que lo confirmes."
        }
        (Lang::En, K::AddFolderHint) => {
            "Add Desktop, Downloads or wherever you keep your files; nothing moves until you confirm."
        }
        (Lang::Es, K::UndoConfirmTitle) => "Deshacer este lote",
        (Lang::En, K::UndoConfirmTitle) => "Undo this batch",
        (Lang::Es, K::UndoConfirmBody) => {
            "Se devolverían {files} archivos a su sitio original. Los que ya no estén o cuyo hueco esté ocupado se quedan como están."
        }
        (Lang::En, K::UndoConfirmBody) => {
            "{files} files would return to their original place. Those that are gone or whose spot is taken stay as they are."
        }
        (Lang::Es, K::Undone) => "Deshecho",
        (Lang::En, K::Undone) => "Undone",
        (Lang::Es, K::TrashNote) => {
            "Lo que va a la papelera se recupera desde la Papelera del sistema. Deshacer solo devuelve los movimientos."
        }
        (Lang::En, K::TrashNote) => {
            "Anything sent to the trash is recovered from the system Trash. Undo only moves files back."
        }
        (Lang::Es, K::SkippedNote) => {
            "{files} archivos no se tocan por ser recientes, estar ocultos o ya estar en su sitio."
        }
        (Lang::En, K::SkippedNote) => {
            "{files} files are left alone because they are recent, hidden or already in place."
        }
        (Lang::Es, K::OpenFolder) => "Abrir carpeta",
        (Lang::En, K::OpenFolder) => "Open folder",
        (Lang::Es, K::CategoriesTitle) => "Grupos y tipos de archivo",
        (Lang::En, K::CategoriesTitle) => "Groups and file types",
        (Lang::Es, K::ExtensionHelp) => {
            "Cada tipo de archivo pertenece a un grupo. Si un tipo de archivo no está en ninguna lista, el archivo va a Otros."
        }
        (Lang::En, K::ExtensionHelp) => {
            "Each file type belongs to one group. If a type is in no list, the file goes to Others."
        }
        (Lang::Es, K::LastRun) => "Última vez",
        (Lang::En, K::LastRun) => "Last run",
        (Lang::Es, K::BackgroundLabel) => "Vigilar al abrir la app",
        (Lang::En, K::BackgroundLabel) => "Watch when the app opens",
        (Lang::Es, K::BackgroundHint) => {
            "Si lo activas, al abrir la ventana también arranca el proceso que organiza solo. Puedes detenerlo cuando quieras."
        }
        (Lang::En, K::BackgroundHint) => {
            "When on, opening the window also starts the process that organizes by itself. You can stop it at any time."
        }
        (Lang::Es, K::AutostartLabel) => "Iniciar con Windows",
        (Lang::En, K::AutostartLabel) => "Start with Windows",
        (Lang::Es, K::AutostartHint) => {
            "Arranca solo el vigilante oculto al entrar en Windows. No abre la ventana."
        }
        (Lang::En, K::AutostartHint) => {
            "Starts only the hidden watcher when Windows signs in. It does not open the window."
        }
        (Lang::Es, K::TrayOpen) => "Abrir",
        (Lang::En, K::TrayOpen) => "Open",
        (Lang::Es, K::TrayToggle) => "Pausar/Reanudar vigilancia",
        (Lang::En, K::TrayToggle) => "Pause/Resume watching",
        (Lang::Es, K::TrayOrganize) => "Organizar ahora",
        (Lang::En, K::TrayOrganize) => "Organize now",
        (Lang::Es, K::TrayQuit) => "Salir",
        (Lang::En, K::TrayQuit) => "Quit",
        (Lang::Es, K::WatchRestartHint) => {
            "Los cambios de reglas, carpetas y exclusiones se aplican solos en la siguiente pasada."
        }
        (Lang::En, K::WatchRestartHint) => {
            "Rule, folder and exclusion changes apply automatically on the next pass."
        }
        (Lang::Es, K::HomeReady) => "Todo en orden",
        (Lang::En, K::HomeReady) => "All in place",
        (Lang::Es, K::CategoryStats) => "Archivos por grupo",
        (Lang::En, K::CategoryStats) => "Files by group",
        (Lang::Es, K::SilentHint) => {
            "No muestra avisos de éxito al organizar. Los errores siempre se muestran y el historial queda igual."
        }
        (Lang::En, K::SilentHint) => {
            "Does not show success notices after organizing. Errors always show and history is unchanged."
        }
        (Lang::Es, K::DataTitle) => "Datos y respaldo",
        (Lang::En, K::DataTitle) => "Data and backup",
        (Lang::Es, K::Trash) => "Papelera",
        (Lang::En, K::Trash) => "Trash",
        (Lang::Es, K::EmptyPlanBody) => {
            "Cuando llegue un archivo nuevo aparecerá aquí, con su destino, antes de moverse."
        }
        (Lang::En, K::EmptyPlanBody) => {
            "A new file will show up here with its destination before anything moves."
        }
        (Lang::Es, K::SizeColumn) => "Tamaño",
        (Lang::En, K::SizeColumn) => "Size",
        (Lang::Es, K::ExcludeHint) => {
            "Separadas por comas. Se comparan con el nombre de la carpeta."
        }
        (Lang::En, K::ExcludeHint) => "Comma separated. Compared against the folder name.",
        (Lang::Es, K::BuiltWith) => "egui y Rust, sin webview",
        (Lang::En, K::BuiltWith) => "egui and Rust, no webview",
        (Lang::Es, K::LanguageSpanish) => "Español",
        (Lang::En, K::LanguageSpanish) => "Spanish",
        (Lang::Es, K::KindRegex) => "Expresion regular",
        (Lang::En, K::KindRegex) => "Regular expression",
        (Lang::Es, K::RuleNameRequired) => "Ponle un nombre a la regla antes de guardarla.",
        (Lang::En, K::RuleNameRequired) => "Give the rule a name before saving it.",
        (Lang::Es, K::RuleValueRequired) => {
            "La regla necesita un valor válido. Para mover o renombrar, el destino no puede estar vacío."
        }
        (Lang::En, K::RuleValueRequired) => {
            "The rule needs a valid value. For move or rename the destination cannot be empty."
        }
        (Lang::Es, K::LanguageEnglish) => "English",
        (Lang::En, K::LanguageEnglish) => "English",
        (Lang::Es, K::TextSizeLabel) => "Tamaño del texto",
        (Lang::En, K::TextSizeLabel) => "Text size",
        (Lang::Es, K::TextSmall) => "Pequeño (12)",
        (Lang::En, K::TextSmall) => "Small (12)",
        (Lang::Es, K::TextNormal) => "Normal (13)",
        (Lang::En, K::TextNormal) => "Normal (13)",
        (Lang::Es, K::TextLarge) => "Grande (15)",
        (Lang::En, K::TextLarge) => "Large (15)",
        (Lang::Es, K::DensityLabel) => "Densidad de filas",
        (Lang::En, K::DensityLabel) => "Row density",
        (Lang::Es, K::DensityCompact) => "Compacta",
        (Lang::En, K::DensityCompact) => "Compact",
        (Lang::Es, K::DensityComfortable) => "Cómoda",
        (Lang::En, K::DensityComfortable) => "Comfortable",
        (Lang::Es, K::AccentCustom) => "Color personalizado",
        (Lang::En, K::AccentCustom) => "Custom color",
        (Lang::Es, K::AccentHexHint) => "#rrggbb",
        (Lang::En, K::AccentHexHint) => "#rrggbb",
        (Lang::Es, K::AccentInvalid) => {
            "El color debe tener el formato #rrggbb, por ejemplo #2563eb."
        }
        (Lang::En, K::AccentInvalid) => {
            "The color must use the #rrggbb format, for example #2563eb."
        }
        (Lang::Es, K::AccentFillWarn) => {
            "El relleno no llega a 4,5:1 con texto blanco. Los botones usarán texto oscuro y el texto sobre el fondo se ajusta solo."
        }
        (Lang::En, K::AccentFillWarn) => {
            "The fill does not reach 4.5:1 with white text. Buttons will use dark text and the text on the background adjusts itself."
        }
        (Lang::Es, K::AccentBlue) => "Azul",
        (Lang::En, K::AccentBlue) => "Blue",
        (Lang::Es, K::AccentGreen) => "Verde",
        (Lang::En, K::AccentGreen) => "Green",
        (Lang::Es, K::AccentPurple) => "Violeta",
        (Lang::En, K::AccentPurple) => "Purple",
        (Lang::Es, K::AccentAmber) => "Ámbar",
        (Lang::En, K::AccentAmber) => "Amber",
        (Lang::Es, K::ApplyColor) => "Aplicar",
        (Lang::En, K::ApplyColor) => "Apply",
        (Lang::Es, K::ProfileTitle) => "Perfiles",
        (Lang::En, K::ProfileTitle) => "Profiles",
        (Lang::Es, K::ProfileGeneral) => "General",
        (Lang::En, K::ProfileGeneral) => "General",
        (Lang::Es, K::ProfileStudent) => "Estudiante",
        (Lang::En, K::ProfileStudent) => "Student",
        (Lang::Es, K::ProfileDeveloper) => "Programador",
        (Lang::En, K::ProfileDeveloper) => "Developer",
        (Lang::Es, K::ProfilePhotographer) => "Fotógrafo",
        (Lang::En, K::ProfilePhotographer) => "Photographer",
        (Lang::Es, K::ProfileApply) => "Aplicar perfil",
        (Lang::En, K::ProfileApply) => "Apply profile",
        (Lang::Es, K::ProfilePreviewTitle) => "Vista previa del perfil",
        (Lang::En, K::ProfilePreviewTitle) => "Profile preview",
        (Lang::Es, K::ProfileApplied) => "Perfil aplicado.",
        (Lang::En, K::ProfileApplied) => "Profile applied.",
        (Lang::Es, K::DateSourceLabel) => "Fecha de las subcarpetas",
        (Lang::En, K::DateSourceLabel) => "Subfolder date",
        (Lang::Es, K::DateModified) => "Modificación del archivo",
        (Lang::En, K::DateModified) => "File modified",
        (Lang::Es, K::DateCreated) => "Creación del archivo",
        (Lang::En, K::DateCreated) => "File created",
        (Lang::Es, K::DateToday) => "Hoy",
        (Lang::En, K::DateToday) => "Today",
        (Lang::Es, K::ExtDuplicateWarn) => {
            "Esa tipo de archivo ya está en otra grupo. La primera grupo de la lista gana."
        }
        (Lang::En, K::ExtDuplicateWarn) => {
            "That extension already belongs to another category. The first category in the list wins."
        }
        (Lang::Es, K::RegexUnavailable) => {
            "Esta regla usa expresión regular, pero esta compilación no incluye esa función. No coincidirá con nada."
        }
        (Lang::En, K::RegexUnavailable) => {
            "This rule uses a regular expression, but this build does not include that feature. It will never match."
        }
        (Lang::Es, K::RegexUnavailableCondition) => {
            "Esta compilación no incluye expresiones regulares. Elige otra condición."
        }
        (Lang::En, K::RegexUnavailableCondition) => {
            "This build does not include regular expressions. Choose another condition."
        }
        (Lang::Es, K::RegexInvalid) => "La expresión regular no es válida.",
        (Lang::En, K::RegexInvalid) => "The regular expression is not valid.",
        (Lang::Es, K::CategoryNew) => "Nuevo grupo",
        (Lang::En, K::CategoryNew) => "New group",
        (Lang::Es, K::CategoryNameLabel) => "Nombre del grupo",
        (Lang::En, K::CategoryNameLabel) => "Name",
        (Lang::Es, K::CategoryFolderLabel) => "Se guardan en",
        (Lang::En, K::CategoryFolderLabel) => "Saved in",
        (Lang::Es, K::CategoryColorLabel) => "Color en la gráfica",
        (Lang::En, K::CategoryColorLabel) => "Chart color",
        (Lang::Es, K::CategoryColorNone) => "Sin color",
        (Lang::En, K::CategoryColorNone) => "No color",
        (Lang::Es, K::CategoryDeleteTitle) => "Borrar grupo",
        (Lang::En, K::CategoryDeleteTitle) => "Delete group",
        (Lang::Es, K::CategoryDeleteBody) => {
            "Se quitará el grupo y sus tipos de archivo. Los archivos ya organizados no se tocan. Su carpeta se excluye para que el próximo escaneo no los reorganice."
        }
        (Lang::En, K::CategoryDeleteBody) => {
            "The group and its file types will be removed. Files already organized are not touched. Its folder is excluded so the next scan does not reorganize them."
        }
        (Lang::Es, K::CategoryFallbackLocked) => "El grupo de reserva no se puede borrar.",
        (Lang::En, K::CategoryFallbackLocked) => "The fallback group cannot be deleted.",
        (Lang::Es, K::CategoryFallbackHint) => {
            "Recoge los archivos que no encajan en ningún otro grupo."
        }
        (Lang::En, K::CategoryFallbackHint) => "It collects files that fit nowhere else.",
        (Lang::Es, K::FolderProblemEmpty) => "El nombre no puede estar vacío.",
        (Lang::En, K::FolderProblemEmpty) => "The name cannot be empty.",
        (Lang::Es, K::FolderProblemInvalid) => "No puede contener < > : \" / \\ | ? *.",
        (Lang::En, K::FolderProblemInvalid) => "It cannot contain < > : \" / \\ | ? *.",
        (Lang::Es, K::FolderProblemReserved) => "Es un nombre reservado de Windows.",
        (Lang::En, K::FolderProblemReserved) => "It is a reserved Windows name.",
        (Lang::Es, K::FolderProblemTrailing) => "No puede terminar en punto ni en espacio.",
        (Lang::En, K::FolderProblemTrailing) => "It cannot end with a dot or a space.",
        (Lang::Es, K::FolderProblemTooLong) => "Máximo 120 caracteres.",
        (Lang::En, K::FolderProblemTooLong) => "120 characters at most.",
        (Lang::Es, K::FolderProblemDuplicate) => "Ya hay otra grupo con esa carpeta.",
        (Lang::En, K::FolderProblemDuplicate) => "Another group already uses that folder.",
        (Lang::Es, K::FolderRenameTitle) => "Cambiar la carpeta destino",
        (Lang::En, K::FolderRenameTitle) => "Change destination folder",
        (Lang::Es, K::FolderRenameBody) => {
            "¿Muevo también el contenido que ya existe en la carpeta anterior?"
        }
        (Lang::En, K::FolderRenameBody) => "Should the content already in the old folder move too?",
        (Lang::Es, K::FolderMoveContent) => "Mover contenido",
        (Lang::En, K::FolderMoveContent) => "Move content",
        (Lang::Es, K::FolderLeaveContent) => "Dejarlo",
        (Lang::En, K::FolderLeaveContent) => "Leave it",
        (Lang::Es, K::FolderMoved) => "Contenido movido.",
        (Lang::En, K::FolderMoved) => "Content moved.",
        (Lang::Es, K::ConditionsTitle) => "Condiciones",
        (Lang::En, K::ConditionsTitle) => "Conditions",
        (Lang::Es, K::ConditionAdd) => "Añadir condición",
        (Lang::En, K::ConditionAdd) => "Add condition",
        (Lang::Es, K::ConditionAll) => "Todas (Y)",
        (Lang::En, K::ConditionAll) => "All (AND)",
        (Lang::Es, K::ConditionAny) => "Alguna (O)",
        (Lang::En, K::ConditionAny) => "Any (OR)",
        (Lang::Es, K::ConditionKind) => "Tipo",
        (Lang::En, K::ConditionKind) => "Type",
        (Lang::Es, K::ConditionValue) => "Valor",
        (Lang::En, K::ConditionValue) => "Value",
        (Lang::Es, K::ConditionDays) => "Días",
        (Lang::En, K::ConditionDays) => "Days",
        (Lang::Es, K::ConditionUnit) => "Unidad",
        (Lang::En, K::ConditionUnit) => "Unit",
        (Lang::Es, K::ConditionRealImage) => "Imagen",
        (Lang::En, K::ConditionRealImage) => "Image",
        (Lang::Es, K::ConditionRealVideo) => "Video",
        (Lang::En, K::ConditionRealVideo) => "Video",
        (Lang::Es, K::ConditionRealAudio) => "Audio",
        (Lang::En, K::ConditionRealAudio) => "Audio",
        (Lang::Es, K::ConditionRealDocument) => "Documento",
        (Lang::En, K::ConditionRealDocument) => "Document",
        (Lang::Es, K::ConditionRealArchive) => "Comprimido",
        (Lang::En, K::ConditionRealArchive) => "Archive",
        (Lang::Es, K::ConditionRealText) => "Texto",
        (Lang::En, K::ConditionRealText) => "Text",
        (Lang::Es, K::ConditionRealFont) => "Fuente",
        (Lang::En, K::ConditionRealFont) => "Font",
        (Lang::Es, K::ConditionRealProgram) => "Programa",
        (Lang::En, K::ConditionRealProgram) => "Program",
        (Lang::Es, K::ConditionRealBook) => "Libro",
        (Lang::En, K::ConditionRealBook) => "Book",
        (Lang::Es, K::KindNameContains) => "El nombre contiene",
        (Lang::En, K::KindNameContains) => "Name contains",
        (Lang::Es, K::KindNameStarts) => "El nombre empieza por",
        (Lang::En, K::KindNameStarts) => "Name starts with",
        (Lang::Es, K::KindNameEnds) => "El nombre termina en",
        (Lang::En, K::KindNameEnds) => "Name ends with",
        (Lang::Es, K::KindMaxSize) => "Tamaño máximo",
        (Lang::En, K::KindMaxSize) => "Maximum size",
        (Lang::Es, K::KindNewer) => "Más reciente que (días)",
        (Lang::En, K::KindNewer) => "Newer than (days)",
        (Lang::Es, K::KindCreatedOlder) => "Creación: más antiguo que (días)",
        (Lang::En, K::KindCreatedOlder) => "Created: older than (days)",
        (Lang::Es, K::KindCreatedNewer) => "Creación: más reciente que (días)",
        (Lang::En, K::KindCreatedNewer) => "Created: newer than (days)",
        (Lang::Es, K::KindRealType) => "Tipo real por firma",
        (Lang::En, K::KindRealType) => "Real type by signature",
        (Lang::Es, K::KindSourceFolder) => "Carpeta de origen",
        (Lang::En, K::KindSourceFolder) => "Source folder",
        (Lang::Es, K::ActionCopy) => "Copiar",
        (Lang::En, K::ActionCopy) => "Copy",
        (Lang::Es, K::ActionMoveRename) => "Mover y renombrar",
        (Lang::En, K::ActionMoveRename) => "Move and rename",
        (Lang::Es, K::ActionCopyRename) => "Copiar y renombrar",
        (Lang::En, K::ActionCopyRename) => "Copy and rename",
        (Lang::Es, K::ActionIgnore) => "Ignorar",
        (Lang::En, K::ActionIgnore) => "Ignore",
        (Lang::Es, K::RuleSubfolderLabel) => "Subcarpeta propia",
        (Lang::En, K::RuleSubfolderLabel) => "Own subfolder",
        (Lang::Es, K::RuleSubfolderHint) => "Año/mes y tipo solo para esta regla.",
        (Lang::En, K::RuleSubfolderHint) => "Year/month and type for this rule only.",
        (Lang::Es, K::RuleTemplate) => "Plantilla",
        (Lang::En, K::RuleTemplate) => "Template",
        (Lang::Es, K::RuleTemplateHint) => "{año} {mes} {dia} {original} {contador} {grupo} {ext}",
        (Lang::En, K::RuleTemplateHint) => "{año} {mes} {dia} {original} {contador} {grupo} {ext}",
        (Lang::Es, K::RuleSameFolderHint) => {
            "Si la carpeta destino es la misma del archivo, no se renombra: así no se repite en cada pasada."
        }
        (Lang::En, K::RuleSameFolderHint) => {
            "If the destination is the file's own folder, it is not renamed, so it does not repeat on every pass."
        }
        (Lang::Es, K::RuleEdit) => "Editar",
        (Lang::En, K::RuleEdit) => "Edit",
        (Lang::Es, K::RuleSave) => "Guardar regla",
        (Lang::En, K::RuleSave) => "Save rule",
        (Lang::Es, K::RuleUpdate) => "Guardar cambios",
        (Lang::En, K::RuleUpdate) => "Save changes",
        (Lang::Es, K::RuleCancel) => "Cancelar",
        (Lang::En, K::RuleCancel) => "Cancel",
        (Lang::Es, K::RuleConditionsEmpty) => "Sin condiciones: la regla no se aplica.",
        (Lang::En, K::RuleConditionsEmpty) => "No conditions: the rule does not apply.",
        (Lang::Es, K::RuleTester) => "Probador de reglas",
        (Lang::En, K::RuleTester) => "Rule tester",
        (Lang::Es, K::RuleTesterName) => "Nombre de archivo",
        (Lang::En, K::RuleTesterName) => "File name",
        (Lang::Es, K::RuleTesterSize) => "Tamaño (KB)",
        (Lang::En, K::RuleTesterSize) => "Size (KB)",
        (Lang::Es, K::RuleTesterDays) => "Antigüedad (días)",
        (Lang::En, K::RuleTesterDays) => "Age (days)",
        (Lang::Es, K::RuleTesterMatch) => "Lo captura",
        (Lang::En, K::RuleTesterMatch) => "Matched by",
        (Lang::Es, K::RuleTesterNoMatch) => "Ninguna regla ni grupo lo captura.",
        (Lang::En, K::RuleTesterNoMatch) => "No rule or group catches it.",
        (Lang::Es, K::RuleTesterLeave) => "Se queda donde está.",
        (Lang::En, K::RuleTesterLeave) => "It stays where it is.",
        (Lang::Es, K::RuleTesterRealTypeNote) => {
            "El tipo real por firma no se comprueba en el probador: hace falta leer el archivo."
        }
        (Lang::En, K::RuleTesterRealTypeNote) => {
            "The real type by signature is not checked in the tester: the file has to be read."
        }
        (Lang::Es, K::RootOverrides) => "Ajustes propios de esta carpeta",
        (Lang::En, K::RootOverrides) => "Settings for this folder",
        (Lang::Es, K::RootOwnSubfolders) => "Subcarpetas propias",
        (Lang::En, K::RootOwnSubfolders) => "Own subfolders",
        (Lang::Es, K::RootExcludeFiles) => "Excluir archivos (glob, separados por comas)",
        (Lang::En, K::RootExcludeFiles) => "Exclude files (glob, comma separated)",
        (Lang::Es, K::RootExcludeDirs) => "Excluir carpetas (glob, separados por comas)",
        (Lang::En, K::RootExcludeDirs) => "Exclude folders (glob, comma separated)",
        (Lang::Es, K::RootOwnRules) => "Reglas y grupos propios",
        (Lang::En, K::RootOwnRules) => "Own rules and groups",
        (Lang::Es, K::RootOwnHint) => "Si lo dejas apagado se usan los ajustes generales.",
        (Lang::En, K::RootOwnHint) => "When off, the global settings are used.",
        (Lang::Es, K::RootEditRules) => "Editar reglas de esta carpeta",
        (Lang::En, K::RootEditRules) => "Edit this folder's rules",
        (Lang::Es, K::ScopeLabel) => "¿En qué carpetas?",
        (Lang::En, K::ScopeLabel) => "Which folders?",
        (Lang::Es, K::ScopeGlobal) => "Todas mis carpetas",
        (Lang::En, K::ScopeGlobal) => "All my folders",
        (Lang::Es, K::TempExtensionsLabel) => "Tipos de archivo temporales",
        (Lang::En, K::TempExtensionsLabel) => "Temporary extensions",
        (Lang::Es, K::SkipDirsLabel) => "Carpetas omitidas",
        (Lang::En, K::SkipDirsLabel) => "Skipped folders",
        (Lang::Es, K::SystemFoldersNote) => {
            "Las carpetas del sistema (Windows, Program Files, AppData...) se omiten siempre y no se pueden quitar."
        }
        (Lang::En, K::SystemFoldersNote) => {
            "System folders (Windows, Program Files, AppData...) are always skipped and cannot be removed."
        }
        (Lang::Es, K::ExportConfig) => "Exportar configuración",
        (Lang::En, K::ExportConfig) => "Export configuration",
        (Lang::Es, K::ImportConfig) => "Importar configuración",
        (Lang::En, K::ImportConfig) => "Import configuration",
        (Lang::Es, K::ImportDone) => "Configuración importada.",
        (Lang::En, K::ImportDone) => "Configuration imported.",
        (Lang::Es, K::ImportInvalid) => {
            "El archivo no es una configuración válida. No se cambió nada."
        }
        (Lang::En, K::ImportInvalid) => {
            "The file is not a valid configuration. Nothing was changed."
        }
        (Lang::Es, K::ImportPreviewTitle) => "Importar configuración",
        (Lang::En, K::ImportPreviewTitle) => "Import configuration",
        (Lang::Es, K::ImportPreviewBody) => {
            "Se reemplazarán todos los ajustes actuales. Revisa las carpetas antes de aceptar."
        }
        (Lang::En, K::ImportPreviewBody) => {
            "All current settings will be replaced. Check the folders before accepting."
        }
        (Lang::Es, K::ConfigExported) => "Configuración exportada.",
        (Lang::En, K::ConfigExported) => "Configuration exported.",
        (Lang::Es, K::FolderAlreadyAdded) => "Esa carpeta ya está en la lista.",
        (Lang::En, K::FolderAlreadyAdded) => "That folder is already in the list.",
        (Lang::Es, K::CopiedNotice) => "Copias creadas.",
        (Lang::En, K::CopiedNotice) => "Copies created.",
        (Lang::Es, K::UndoCopyNote) => {
            "Al deshacer, las copias de este lote se envían a la papelera. Los originales no se tocan."
        }
        (Lang::En, K::UndoCopyNote) => {
            "When undone, the copies in this batch go to the trash. The originals are not touched."
        }
        (Lang::Es, K::Apply) => "Aplicar",
        (Lang::En, K::Apply) => "Apply",
        (Lang::Es, K::RuleConditionRequired) => {
            "Añade al menos una condición para que la regla no se aplique a todo."
        }
        (Lang::En, K::RuleConditionRequired) => {
            "Add at least one condition so the rule does not apply to everything."
        }
        (Lang::Es, K::RuleEnabled) => "Regla activa",
        (Lang::En, K::RuleEnabled) => "Rule on",
        (Lang::Es, K::CategoryEnabled) => "Grupo activo",
        (Lang::En, K::CategoryEnabled) => "Group active",
        (Lang::Es, K::RootEnabled) => "Carpeta activa",
        (Lang::En, K::RootEnabled) => "Folder on",
        (Lang::Es, K::DataFolderTitle) => "Datos y archivos",
        (Lang::En, K::DataFolderTitle) => "Data and files",
        (Lang::Es, K::NoData) => "Aún no hay datos.",
        (Lang::En, K::NoData) => "No data yet.",
        (Lang::Es, K::ExclusionsTitle) => "Exclusiones",
        (Lang::En, K::ExclusionsTitle) => "Exclusions",
        (Lang::Es, K::SourceFolderHint) => "Carpeta dentro de la raíz",
        (Lang::En, K::SourceFolderHint) => "Folder inside the root",
        (Lang::Es, K::ModeTitle) => "Modo de la aplicación",
        (Lang::En, K::ModeTitle) => "App mode",
        (Lang::Es, K::ModeSimple) => "Sencillo",
        (Lang::En, K::ModeSimple) => "Simple",
        (Lang::Es, K::ModeAdvanced) => "Avanzado",
        (Lang::En, K::ModeAdvanced) => "Advanced",
        (Lang::Es, K::ModeHint) => {
            "En Sencillo ves lo necesario para ordenar. Avanzado añade reglas especiales, vigilancia y opciones técnicas."
        }
        (Lang::En, K::ModeHint) => {
            "Simple shows what you need to organize. Advanced adds special rules, watching and technical options."
        }
        (Lang::Es, K::SectionStyle) => "Estilo",
        (Lang::En, K::SectionStyle) => "Style",
        (Lang::Es, K::StyleTitle) => "Cómo se guardan los archivos",
        (Lang::En, K::StyleTitle) => "How files are stored",
        (Lang::Es, K::StyleHint) => {
            "Elige si dentro de cada grupo quieres una carpeta por tipo de archivo o una carpeta por tipo y fecha."
        }
        (Lang::En, K::StyleHint) => {
            "Choose a folder per file type, or a folder per type and date, inside each group."
        }
        (Lang::Es, K::StyleByType) => "Por tipo de archivo",
        (Lang::En, K::StyleByType) => "By file type",
        (Lang::Es, K::StyleByTypeAndDate) => "Por tipo y fecha",
        (Lang::En, K::StyleByTypeAndDate) => "By type and date",
        (Lang::Es, K::ShortcutsHint) => "Añade una carpeta conocida con un clic:",
        (Lang::En, K::ShortcutsHint) => "Add a familiar folder with one click:",
        (Lang::Es, K::FolderDownloads) => "Descargas",
        (Lang::En, K::FolderDownloads) => "Downloads",
        (Lang::Es, K::FolderDesktop) => "Escritorio",
        (Lang::En, K::FolderDesktop) => "Desktop",
        (Lang::Es, K::FolderDocuments) => "Documentos",
        (Lang::En, K::FolderDocuments) => "Documents",
        (Lang::Es, K::FolderActive) => "Activa",
        (Lang::En, K::FolderActive) => "Active",
        (Lang::Es, K::FolderPaused) => "Pausada",
        (Lang::En, K::FolderPaused) => "Paused",
        (Lang::Es, K::FolderAdded) => "Carpeta añadida.",
        (Lang::En, K::FolderAdded) => "Folder added.",
        (Lang::Es, K::FolderRemoved) => "Carpeta quitada. Puedes deshacerlo.",
        (Lang::En, K::FolderRemoved) => "Folder removed. You can undo it.",
        (Lang::Es, K::FolderEmptyShortcuts) => {
            "Todavía no hay carpetas. Añade una o empieza por una de estas."
        }
        (Lang::En, K::FolderEmptyShortcuts) => {
            "No folders yet. Add one, or start with one of these."
        }
        (Lang::Es, K::GroupTitle) => "Grupos y tipos de archivo",
        (Lang::En, K::GroupTitle) => "Groups and file types",
        (Lang::Es, K::GroupHint) => {
            "Cada archivo va al grupo que corresponde a su tipo. Si un tipo no está en ningún grupo, va a Otros."
        }
        (Lang::En, K::GroupHint) => {
            "Each file goes to the group for its type. A type in no group goes to Others."
        }
        (Lang::Es, K::GroupCount) => "{count} tipos de archivo",
        (Lang::En, K::GroupCount) => "{count} file types",
        (Lang::Es, K::GroupCountOne) => "1 tipo de archivo",
        (Lang::En, K::GroupCountOne) => "1 file type",
        (Lang::Es, K::GroupActive) => "Activo",
        (Lang::En, K::GroupActive) => "Active",
        (Lang::Es, K::GroupActiveHint) => {
            "Activo significa que los archivos de estos tipos van a este grupo."
        }
        (Lang::En, K::GroupActiveHint) => "Active means files of these types go to this group.",
        (Lang::Es, K::GroupPausedHint) => {
            "Si lo apagas, este grupo deja de usarse y sus archivos van a Otros."
        }
        (Lang::En, K::GroupPausedHint) => {
            "When off, this group is not used and its files go to Others."
        }
        (Lang::Es, K::GroupName) => "Nombre del grupo",
        (Lang::En, K::GroupName) => "Group name",
        (Lang::Es, K::GroupTypes) => "Tipos de archivo",
        (Lang::En, K::GroupTypes) => "File types",
        (Lang::Es, K::GroupTypesEmpty) => "Este grupo todavía no tiene tipos.",
        (Lang::En, K::GroupTypesEmpty) => "This group has no file types yet.",
        (Lang::Es, K::GroupAddType) => "Añadir tipo",
        (Lang::En, K::GroupAddType) => "Add type",
        (Lang::Es, K::GroupTypeHint) => "Ej: .psd",
        (Lang::En, K::GroupTypeHint) => "E.g. .psd",
        (Lang::Es, K::GroupColor) => "Color del grupo",
        (Lang::En, K::GroupColor) => "Group color",
        (Lang::Es, K::GroupColorAccent) => "Usar el color de la app",
        (Lang::En, K::GroupColorAccent) => "Use the app color",
        (Lang::Es, K::GroupFolder) => "Se guardan en",
        (Lang::En, K::GroupFolder) => "Saved in",
        (Lang::Es, K::GroupFolderHint) => {
            "Cambiar el nombre de la carpeta es una opción avanzada. El grupo y la carpeta pueden llamarse distinto."
        }
        (Lang::En, K::GroupFolderHint) => {
            "Renaming the folder is an advanced option. The group and folder can have different names."
        }
        (Lang::Es, K::GroupAdvanced) => "Más opciones del grupo",
        (Lang::En, K::GroupAdvanced) => "More group options",
        (Lang::Es, K::GroupRestore) => "Restaurar grupos por defecto",
        (Lang::En, K::GroupRestore) => "Restore default groups",
        (Lang::Es, K::GroupRestoreDone) => "Grupos restaurados. Puedes deshacerlo.",
        (Lang::En, K::GroupRestoreDone) => "Groups restored. You can undo it.",
        (Lang::Es, K::GroupDelete) => "Borrar grupo",
        (Lang::En, K::GroupDelete) => "Delete group",
        (Lang::Es, K::GroupDeleted) => "Grupo borrado. Puedes deshacerlo.",
        (Lang::En, K::GroupDeleted) => "Group deleted. You can undo it.",
        (Lang::Es, K::GroupAdd) => "Añadir grupo",
        (Lang::En, K::GroupAdd) => "Add group",
        (Lang::Es, K::GroupNoDelete) => "El grupo Otros no se puede borrar.",
        (Lang::En, K::GroupNoDelete) => "The Others group cannot be deleted.",
        (Lang::Es, K::GroupTypeInUse) => "Ese tipo ya está en el grupo {group}.",
        (Lang::En, K::GroupTypeInUse) => "That type is already in the {group} group.",
        (Lang::Es, K::GroupNewName) => "Grupo {number}",
        (Lang::En, K::GroupNewName) => "Group {number}",
        (Lang::Es, K::RuleExampleName) => "factura",
        (Lang::En, K::RuleExampleName) => "invoice",
        (Lang::Es, K::RuleExampleSize) => "10",
        (Lang::En, K::RuleExampleSize) => "10",
        (Lang::Es, K::RuleExampleDays) => "30",
        (Lang::En, K::RuleExampleDays) => "30",
        (Lang::Es, K::RuleSpecialTitle) => "Reglas especiales",
        (Lang::En, K::RuleSpecialTitle) => "Special rules",
        (Lang::Es, K::RuleListTitle) => "Tus reglas",
        (Lang::En, K::RuleListTitle) => "Your rules",
        (Lang::Es, K::RuleSpecialHint) => {
            "Un archivo factura.pdf irá a Documentos. Las reglas especiales se aplican antes que los grupos."
        }
        (Lang::En, K::RuleSpecialHint) => {
            "A factura.pdf file will go to Documents. Special rules run before groups."
        }
        (Lang::Es, K::RuleWhen) => "Cuando",
        (Lang::En, K::RuleWhen) => "When",
        (Lang::Es, K::RuleThen) => "→",
        (Lang::En, K::RuleThen) => "→",
        (Lang::Es, K::RuleNameContains) => "el nombre contiene",
        (Lang::En, K::RuleNameContains) => "the name contains",
        (Lang::Es, K::RuleTypeIs) => "es de tipo",
        (Lang::En, K::RuleTypeIs) => "is type",
        (Lang::Es, K::RuleWeighsMore) => "pesa más de (MB)",
        (Lang::En, K::RuleWeighsMore) => "is larger than (MB)",
        (Lang::Es, K::RuleOlderThan) => "tiene más de (días)",
        (Lang::En, K::RuleOlderThan) => "is older than (days)",
        (Lang::Es, K::RuleDestination) => "mover a",
        (Lang::En, K::RuleDestination) => "move to",
        (Lang::Es, K::RuleMoreOptions) => "Más opciones de esta regla",
        (Lang::En, K::RuleMoreOptions) => "More options for this rule",
        (Lang::Es, K::LivePreviewTitle) => "Así quedaría",
        (Lang::En, K::LivePreviewTitle) => "This is how it would end up",
        (Lang::Es, K::LivePreviewHint) => {
            "Escribe un nombre de ejemplo. Se usa el destino que darían las reglas actuales."
        }
        (Lang::En, K::LivePreviewHint) => {
            "Type an example name. It shows where the current rules would send it."
        }
        (Lang::Es, K::LivePreviewGoes) => "{name} iría a {destination}.",
        (Lang::En, K::LivePreviewGoes) => "{name} would go to {destination}.",
        (Lang::Es, K::LivePreviewStays) => "{name} se quedaría donde está.",
        (Lang::En, K::LivePreviewStays) => "{name} would stay where it is.",
        (Lang::Es, K::LivePreviewTrash) => "{name} iría a la papelera.",
        (Lang::En, K::LivePreviewTrash) => "{name} would go to the trash.",
        (Lang::Es, K::TooltipWatch) => {
            "Vigilancia: mientras está activa, los archivos nuevos se ordenan solos. Ejemplo: descargas un PDF y aparece en Documentos."
        }
        (Lang::En, K::TooltipWatch) => {
            "Watching: while on, new files are organized by themselves. Example: you download a PDF and it appears in Documents."
        }
        (Lang::Es, K::TooltipScope) => {
            "¿En qué carpetas?: elige si la regla vale para todas tus carpetas o solo para una. Ejemplo: solo Descargas."
        }
        (Lang::En, K::TooltipScope) => {
            "Which folders?: choose whether the rule applies to all your folders or just one. Example: Downloads only."
        }
        (Lang::Es, K::TooltipSubfolders) => {
            "Subcarpetas: guarda cada tipo en su propia carpeta dentro del grupo. Ejemplo: Documentos/PDF/."
        }
        (Lang::En, K::TooltipSubfolders) => {
            "Subfolders: keep each type in its own folder inside the group. Example: Documents/PDF/."
        }
        (Lang::Es, K::TooltipSpecialRule) => {
            "Regla especial: una excepción que se aplica antes que los grupos. Ejemplo: todo lo que contenga factura va a Documentos."
        }
        (Lang::En, K::TooltipSpecialRule) => {
            "Special rule: an exception applied before groups. Example: anything containing factura goes to Documents."
        }
        (Lang::Es, K::OnboardingTitle) => "Vamos a ordenar tus archivos",
        (Lang::En, K::OnboardingTitle) => "Let's organize your files",
        (Lang::Es, K::OnboardingStep) => "Paso {step} de 3",
        (Lang::En, K::OnboardingStep) => "Step {step} of 3",
        (Lang::Es, K::OnboardingStep1Title) => "Elige una carpeta",
        (Lang::En, K::OnboardingStep1Title) => "Choose a folder",
        (Lang::Es, K::OnboardingStep1Body) => {
            "Empieza por Descargas o Escritorio. Nada se mueve hasta que lo confirmes."
        }
        (Lang::En, K::OnboardingStep1Body) => {
            "Start with Downloads or Desktop. Nothing moves until you confirm."
        }
        (Lang::Es, K::OnboardingStep2Title) => "Elige el estilo",
        (Lang::En, K::OnboardingStep2Title) => "Choose the style",
        (Lang::Es, K::OnboardingStep2Body) => {
            "Puedes guardar una carpeta por tipo de archivo, o una por tipo y fecha."
        }
        (Lang::En, K::OnboardingStep2Body) => {
            "Store one folder per file type, or one per type and date."
        }
        (Lang::Es, K::OnboardingStep3Title) => "Revisa y organiza",
        (Lang::En, K::OnboardingStep3Title) => "Review and organize",
        (Lang::Es, K::OnboardingStep3Body) => {
            "Mira la vista previa. Si todo está bien, organiza; si no, puedes salir sin tocar nada."
        }
        (Lang::En, K::OnboardingStep3Body) => {
            "Check the preview. If it looks right, organize; if not, leave without touching anything."
        }
        (Lang::Es, K::OnboardingPick) => "Añadir carpeta",
        (Lang::En, K::OnboardingPick) => "Add folder",
        (Lang::Es, K::OnboardingNoFolders) => {
            "Añade al menos una carpeta para continuar, o salta la guía."
        }
        (Lang::En, K::OnboardingNoFolders) => {
            "Add at least one folder to continue, or skip the guide."
        }
        (Lang::Es, K::OnboardingFolderAdded) => "Carpeta añadida",
        (Lang::En, K::OnboardingFolderAdded) => "Folder added",
        (Lang::Es, K::OnboardingPreview) => "Ver vista previa",
        (Lang::En, K::OnboardingPreview) => "See preview",
        (Lang::Es, K::OnboardingFinish) => "Terminar",
        (Lang::En, K::OnboardingFinish) => "Finish",
        (Lang::Es, K::OnboardingSkip) => "Saltar por ahora",
        (Lang::En, K::OnboardingSkip) => "Skip for now",
        (Lang::Es, K::Saved) => "Guardado",
        (Lang::En, K::Saved) => "Saved",
        (Lang::Es, K::UndoDisabledHint) => {
            "Todavía no hay nada que deshacer. Organiza archivos y podrás volver atrás."
        }
        (Lang::En, K::UndoDisabledHint) => {
            "Nothing to undo yet. Organize files and you can go back."
        }
        (Lang::Es, K::OrganizeDisabledHint) => {
            "Primero revisa la vista previa. Cuando haya archivos, podrás organizarlos."
        }
        (Lang::En, K::OrganizeDisabledHint) => {
            "Check the preview first. When there are files, you can organize them."
        }
        (Lang::Es, K::TypePdf) => "PDF",
        (Lang::En, K::TypePdf) => "PDF",
        (Lang::Es, K::TypeWord) => "Word",
        (Lang::En, K::TypeWord) => "Word",
        (Lang::Es, K::TypeExcel) => "Excel",
        (Lang::En, K::TypeExcel) => "Excel",
        (Lang::Es, K::TypePowerPoint) => "PowerPoint",
        (Lang::En, K::TypePowerPoint) => "PowerPoint",
        (Lang::Es, K::TypeImage) => "Imagen",
        (Lang::En, K::TypeImage) => "Image",
        (Lang::Es, K::TypeVideo) => "Vídeo",
        (Lang::En, K::TypeVideo) => "Video",
        (Lang::Es, K::TypeAudio) => "Audio",
        (Lang::En, K::TypeAudio) => "Audio",
        (Lang::Es, K::TypeText) => "Texto",
        (Lang::En, K::TypeText) => "Text",
        (Lang::Es, K::TypeArchive) => "Comprimido",
        (Lang::En, K::TypeArchive) => "Archive",
        (Lang::Es, K::TypeProgram) => "Programa",
        (Lang::En, K::TypeProgram) => "Program",
        (Lang::Es, K::TypeBook) => "Libro",
        (Lang::En, K::TypeBook) => "Book",
        (Lang::Es, K::TypeFont) => "Fuente",
        (Lang::En, K::TypeFont) => "Font",
        (Lang::Es, K::TypeOther) => "Otro",
        (Lang::En, K::TypeOther) => "Other",
        (Lang::Es, K::EmptyHomeTitle) => "Empecemos por una carpeta",
        (Lang::En, K::EmptyHomeTitle) => "Let's start with a folder",
        (Lang::Es, K::EmptyHomeBody) => {
            "Elige dónde guardas tus archivos; nada se mueve hasta que lo confirmes."
        }
        (Lang::En, K::EmptyHomeBody) => {
            "Choose where you keep your files; nothing moves until you confirm it."
        }
        (Lang::Es, K::EmptyPreviewBody) => {
            "Aún no hay nada que mostrar. Añade una carpeta para empezar."
        }
        (Lang::En, K::EmptyPreviewBody) => {
            "There is nothing to show yet. Add a folder to get started."
        }
        (Lang::Es, K::SaveListEmptyHint) => {
            "Todavía no hay actividad. Cuando organices archivos podrás guardar la lista."
        }
        (Lang::En, K::SaveListEmptyHint) => {
            "No activity yet. Once you organize files you can save the list."
        }
        (Lang::Es, K::AutoOrganize) => "Organizar automáticamente las descargas nuevas",
        (Lang::En, K::AutoOrganize) => "Automatically organize new downloads",
        (Lang::Es, K::AutoOrganizeHint) => {
            "Vigila solo las carpetas añadidas. Los ajustes finos están en Avanzado."
        }
        (Lang::En, K::AutoOrganizeHint) => {
            "Watches only the folders you added. Fine tuning lives in Advanced."
        }
        (Lang::Es, K::ProfileDescriptionBasic) => {
            "Grupos y reglas para separar documentos, imágenes y comprimidos."
        }
        (Lang::En, K::ProfileDescriptionBasic) => {
            "Groups and rules to separate documents, images and archives."
        }
        (Lang::Es, K::ProfileDescriptionStudent) => {
            "Apuntes, trabajos y libros por asignatura y por fecha."
        }
        (Lang::En, K::ProfileDescriptionStudent) => {
            "Notes, assignments and books by subject and date."
        }
        (Lang::Es, K::ProfileDescriptionDeveloper) => {
            "Código, dependencias y ejecutables separados del resto."
        }
        (Lang::En, K::ProfileDescriptionDeveloper) => {
            "Code, dependencies and executables kept apart from the rest."
        }
        (Lang::Es, K::ProfileDescriptionPhotographer) => {
            "Fotos y vídeos por año, mes y tipo de archivo."
        }
        (Lang::En, K::ProfileDescriptionPhotographer) => {
            "Photos and videos by year, month and file type."
        }
        (Lang::Es, K::ProfileOnlyMissing) => {
            "Solo se añadirán los grupos y reglas que falten. Lo tuyo no se sobrescribe."
        }
        (Lang::En, K::ProfileOnlyMissing) => {
            "Only missing groups and rules are added. Your own changes are left alone."
        }
        (Lang::Es, K::ProfileNothingNew) => "Este perfil no añade nada nuevo.",
        (Lang::En, K::ProfileNothingNew) => "This profile adds nothing new.",
        (Lang::Es, K::ApplyChanges) => "Aplicar",
        (Lang::En, K::ApplyChanges) => "Apply",
        (Lang::Es, K::DropHere) => "Suelta aquí para añadir la carpeta",
        (Lang::En, K::DropHere) => "Drop here to add the folder",
        (Lang::Es, K::DuplicatesTitle) => "Duplicados",
        (Lang::En, K::DuplicatesTitle) => "Duplicates",
        (Lang::Es, K::DuplicatesIntro) => {
            "Busca archivos repetidos y revisa el resultado antes de tocar nada."
        }
        (Lang::En, K::DuplicatesIntro) => {
            "Find repeated files and review the result before touching anything."
        }
        (Lang::Es, K::DuplicatesIdentical) => {
            "Estos archivos son idénticos. Conservaremos uno de cada grupo."
        }
        (Lang::En, K::DuplicatesIdentical) => {
            "These files are identical. We will keep one from each group."
        }
        (Lang::Es, K::FindDuplicates) => "Buscar duplicados",
        (Lang::En, K::FindDuplicates) => "Find duplicates",
        (Lang::Es, K::DuplicatesScanning) => "Comparando archivos…",
        (Lang::En, K::DuplicatesScanning) => "Comparing files…",
        (Lang::Es, K::DuplicatesCancel) => "Cancelar búsqueda",
        (Lang::En, K::DuplicatesCancel) => "Cancel search",
        (Lang::Es, K::DuplicatesEmpty) => {
            "No se han encontrado archivos idénticos en las carpetas añadidas."
        }
        (Lang::En, K::DuplicatesEmpty) => "No identical files were found in the folders you added.",
        (Lang::Es, K::DuplicatesKeep) => "Conservar",
        (Lang::En, K::DuplicatesKeep) => "Keep",
        (Lang::Es, K::DuplicatesDelete) => "Enviar a la papelera",
        (Lang::En, K::DuplicatesDelete) => "Send to Recycle Bin",
        (Lang::Es, K::DuplicatesFreed) => "Se liberarán",
        (Lang::En, K::DuplicatesFreed) => "Will free up",
        (Lang::Es, K::DuplicatesAction) => "Enviando a la papelera",
        (Lang::En, K::DuplicatesAction) => "Sending to the Recycle Bin",
        (Lang::Es, K::DuplicatesNoSelection) => {
            "Selecciona al menos una copia para enviar a la papelera."
        }
        (Lang::En, K::DuplicatesNoSelection) => {
            "Select at least one copy to send to the Recycle Bin."
        }
        (Lang::Es, K::ProfileSkipped) => "Se han conservado tus grupos y reglas actuales.",
        (Lang::En, K::ProfileSkipped) => "Your current groups and rules were kept.",
        (Lang::Es, K::ViewChanges) => "Ver cambios",
        (Lang::En, K::ViewChanges) => "View changes",
        (Lang::Es, K::StepChooseFolder) => "Elige una carpeta",
        (Lang::En, K::StepChooseFolder) => "Choose a folder",
        (Lang::Es, K::StepReviewFiles) => "Revisa lo que se movería",
        (Lang::En, K::StepReviewFiles) => "Review what would move",
        (Lang::Es, K::StepMoveFiles) => "Mueve los archivos",
        (Lang::En, K::StepMoveFiles) => "Move the files",
        (Lang::Es, K::RuleConditionValueRequired) => "Escribe un valor para la condición.",
        (Lang::En, K::RuleConditionValueRequired) => "Enter a value for the condition.",
        (Lang::Es, K::RuleConditionNumber) => "Escribe un número mayor que cero.",
        (Lang::En, K::RuleConditionNumber) => "Enter a number greater than zero.",
        (Lang::Es, K::RuleDestinationRequired) => "Elige o escribe una carpeta de destino.",
        (Lang::En, K::RuleDestinationRequired) => "Choose or enter a destination folder.",
        (Lang::Es, K::HistoryReviewHint) => {
            "Hay archivos en el origen y en el destino. No se borró nada: revisa ambos y decide cuál conservar."
        }
        (Lang::En, K::HistoryReviewHint) => {
            "Files exist in both the source and the destination. Nothing was deleted: review both and decide which to keep."
        }
        (Lang::Es, K::RestartWatch) => "Reiniciar vigilancia",
        (Lang::En, K::RestartWatch) => "Restart watching",
        (Lang::Es, K::WatchFailed) => {
            "La vigilancia se detuvo varias veces. No se reintentará sola; reiníciala cuando quieras."
        }
        (Lang::En, K::WatchFailed) => {
            "Watching stopped several times. It will not retry on its own; restart it when you are ready."
        }
        (Lang::Es, K::WatchFailedShort) => "Vigilancia detenida",
        (Lang::En, K::WatchFailedShort) => "Watching stopped",
        (Lang::Es, K::WatchRetrying) => "Reintentando vigilancia",
        (Lang::En, K::WatchRetrying) => "Retrying watching",
        (Lang::Es, K::GuiStartFailed) => {
            "No pudimos iniciar la interfaz gráfica. Es probable que estés en Escritorio Remoto, una máquina virtual o que falte aceleración gráfica. Si ya configuraste carpetas, la vigilancia puede seguir funcionando con el modo --watch."
        }
        (Lang::En, K::GuiStartFailed) => {
            "We couldn't start the graphical interface. You may be on Remote Desktop or a virtual machine, or graphics acceleration may be missing. If you already set up folders, watching can keep running with the --watch mode."
        }
        (Lang::Es, K::SwitchOn) => "Activado",
        (Lang::En, K::SwitchOn) => "On",
        (Lang::Es, K::SwitchOff) => "Desactivado",
        (Lang::En, K::SwitchOff) => "Off",
        (Lang::Es, K::StateExpanded) => "Desplegado",
        (Lang::En, K::StateExpanded) => "Expanded",
        (Lang::Es, K::StateCollapsed) => "Contraído",
        (Lang::En, K::StateCollapsed) => "Collapsed",
    }
}

/// Nombre entendible para un tipo de archivo. Los tipos desconocidos se
/// muestran sin el punto, que es lo único que hace falta entender.
pub fn file_type_name(lang: Lang, extension: &str) -> String {
    let ext = extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    let key = match ext.as_str() {
        "pdf" => Some(Key::TypePdf),
        "doc" | "docx" | "odt" | "rtf" => Some(Key::TypeWord),
        "xls" | "xlsx" | "ods" | "csv" => Some(Key::TypeExcel),
        "ppt" | "pptx" | "odp" => Some(Key::TypePowerPoint),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tif" | "tiff" | "svg" | "heic" => {
            Some(Key::TypeImage)
        }
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "flv" => Some(Key::TypeVideo),
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "wma" => Some(Key::TypeAudio),
        "txt" | "md" | "log" => Some(Key::TypeText),
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" => Some(Key::TypeArchive),
        "exe" | "msi" | "dmg" | "pkg" | "deb" | "rpm" | "appimage" => Some(Key::TypeProgram),
        "epub" | "mobi" | "azw3" => Some(Key::TypeBook),
        "ttf" | "otf" | "woff" | "woff2" => Some(Key::TypeFont),
        _ => None,
    };
    key.map(|key| t(lang, key).to_string()).unwrap_or_else(|| {
        if ext.is_empty() {
            t(lang, Key::TypeOther).to_string()
        } else {
            ext.to_ascii_uppercase()
        }
    })
}

/// Resumen de tipos para una fila cerrada. Repite cada nombre entendible una
/// sola vez: PDF, Word, Excel.
pub fn file_types_summary(lang: Lang, extensions: &[String]) -> String {
    let mut names: Vec<String> = extensions
        .iter()
        .map(|ext| file_type_name(lang, ext))
        .collect();
    names.sort();
    names.dedup();
    names.join(", ")
}

pub fn group_count(lang: Lang, count: usize) -> String {
    if count == 1 {
        return t(lang, Key::GroupCountOne).to_string();
    }
    t(lang, Key::GroupCount).replace("{count}", &count.to_string())
}

pub fn onboarding_step(lang: Lang, step: usize) -> String {
    t(lang, Key::OnboardingStep).replace("{step}", &step.to_string())
}

pub fn profile_preview(lang: Lang, categories: &str, rules: &str) -> String {
    match lang {
        Lang::Es => format!(
            "Se reemplazarán los grupos y las reglas actuales. Grupos del perfil: {categories}. Reglas: {rules}."
        ),
        Lang::En => format!(
            "The current groups and rules will be replaced. Profile groups: {categories}. Rules: {rules}."
        ),
    }
}

pub fn import_preview(lang: Lang, roots: usize, categories: usize, rules: usize) -> String {
    match lang {
        Lang::Es => format!(
            "El archivo trae {roots} carpetas, {categories} grupos y {rules} reglas. Se reemplazará todo lo actual."
        ),
        Lang::En => format!(
            "The file has {roots} folders, {categories} groups and {rules} rules. Everything current will be replaced."
        ),
    }
}

pub fn folder_rename_body(lang: Lang, old: &str, new: &str) -> String {
    match lang {
        Lang::Es => format!("{old} → {new}. ¿Muevo también el contenido que ya existe en {old}?"),
        Lang::En => format!("{old} → {new}. Should the content already in {old} move too?"),
    }
}

pub fn folder_left_excluded(lang: Lang, old: &str) -> String {
    match lang {
        Lang::Es => format!(
            "El contenido se quedó en {old}. Esa carpeta se excluye para que el próximo escaneo no lo mueva."
        ),
        Lang::En => format!(
            "The content stayed in {old}. That folder is excluded so the next scan does not move it."
        ),
    }
}

pub fn organized_notice(lang: Lang, moved: usize, copied: usize, trashed: usize) -> String {
    match lang {
        Lang::Es => {
            let mut parts = Vec::new();
            if moved > 0 {
                parts.push(format!("{moved} movidos"));
            }
            if copied > 0 {
                parts.push(format!("{copied} copiados"));
            }
            if trashed > 0 {
                parts.push(format!("{trashed} a la papelera"));
            }
            format!(
                "Listo: {}. Puedes deshacerlo desde el historial o con Ctrl+Z.",
                parts.join(", ")
            )
        }
        Lang::En => {
            let mut parts = Vec::new();
            if moved > 0 {
                parts.push(format!("{moved} moved"));
            }
            if copied > 0 {
                parts.push(format!("{copied} copied"));
            }
            if trashed > 0 {
                parts.push(format!("{trashed} trashed"));
            }
            format!(
                "Done: {}. Undo from history or with Ctrl+Z.",
                parts.join(", ")
            )
        }
    }
}

pub fn cli_copied(lang: Lang, copied: usize) -> String {
    match lang {
        Lang::Es => format!("Se crearon {copied} copias. Los originales siguen donde estaban."),
        Lang::En => format!("Created {copied} copies. The originals are still in place."),
    }
}

pub fn config_exported(lang: Lang, path: &Path) -> String {
    let path = path.display();
    match lang {
        Lang::Es => format!("Configuración exportada a {path}"),
        Lang::En => format!("Configuration exported to {path}"),
    }
}

pub fn folder_moved(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es => format!("Se movieron {files} archivos a la carpeta nueva."),
        Lang::En => format!("Moved {files} files to the new folder."),
    }
}

pub fn local_offset(lang: Lang, offset: &str) -> String {
    match lang {
        Lang::Es => format!("Zona horaria local: {offset}"),
        Lang::En => format!("Local time zone: {offset}"),
    }
}

pub fn error_io(lang: Lang, path: &Path, source: &std::io::Error) -> String {
    let path = path.display();
    match lang {
        Lang::Es => format!(
            "No se pudo acceder a {path}. No se movió nada. Revisa que la carpeta siga existiendo y que tengas permiso sobre ella. Detalle: {source}"
        ),
        Lang::En => format!(
            "Could not access {path}. Nothing was moved. Check that the folder still exists and that you can write to it. Detail: {source}"
        ),
    }
}

pub fn error_clock(lang: Lang) -> &'static str {
    match lang {
        Lang::Es => {
            "No se pudo leer la zona horaria del sistema, así que las fechas no se pueden mostrar con seguridad. La app no va a mover nada hasta resolverlo."
        }
        Lang::En => {
            "The system time zone could not be read, so dates cannot be shown reliably. The app will not move anything until this is fixed."
        }
    }
}

pub fn error_gui(lang: Lang, detail: &str) -> String {
    match lang {
        Lang::Es => format!(
            "La ventana no pudo abrirse. Puede faltar un driver de video actualizado. Detalle: {detail}"
        ),
        Lang::En => format!(
            "The window could not open. The video driver may be out of date. Detail: {detail}"
        ),
    }
}

pub fn error_gui_opengl(lang: Lang, detail: &str) -> String {
    match lang {
        Lang::Es => format!(
            "La ventana no pudo crearse con OpenGL. No se movió ningún archivo. Actualiza el driver de video y evita el Escritorio remoto; si sigue igual, abre una sesión local o prueba otro equipo. Detalle: {detail}"
        ),
        Lang::En => format!(
            "The window could not be created with OpenGL. No file was moved. Update the video driver and avoid Remote Desktop; if it persists, use a local session or another computer. Detail: {detail}"
        ),
    }
}

pub fn error_autostart(lang: Lang, detail: &str) -> String {
    match lang {
        Lang::Es => format!(
            "Windows no aceptó el inicio automático. La app sigue funcionando y no se tocó ningún archivo. Puedes activarlo desde Ajustes o crear la entrada a mano. Detalle: {detail}"
        ),
        Lang::En => format!(
            "Windows did not accept the startup entry. The app still works and no file was touched. You can try again from Settings or add the entry manually. Detail: {detail}"
        ),
    }
}

pub fn error_config(lang: Lang, path: &Path, detail: &str) -> String {
    let path = path.display();
    match lang {
        Lang::Es => format!(
            "El archivo de ajustes {path} no se pudo leer, así que se usan los valores seguros por defecto. Tus ajustes anteriores siguen en el disco. Detalle: {detail}"
        ),
        Lang::En => format!(
            "The settings file {path} could not be read, so safe defaults are in use. Your previous settings are still on disk. Detail: {detail}"
        ),
    }
}

pub fn error_journal(lang: Lang, path: &Path, detail: &str) -> String {
    let path = path.display();
    match lang {
        Lang::Es => format!(
            "El historial en {path} no se pudo escribir. Los archivos ya movidos siguen en su sitio nuevo, pero deshacer puede quedar incompleto. Detalle: {detail}"
        ),
        Lang::En => format!(
            "The history file {path} could not be written. Files already moved stay where they are, but undo may be incomplete. Detail: {detail}"
        ),
    }
}

pub fn error_outside_roots(lang: Lang, path: &Path) -> String {
    let path = path.display();
    match lang {
        Lang::Es => format!(
            "{path} queda fuera de las carpetas que elegiste. Se omitió para no tocar nada ajeno."
        ),
        Lang::En => format!(
            "{path} is outside the folders you chose. It was skipped so nothing else is touched."
        ),
    }
}

pub fn error_protected(lang: Lang, path: &Path) -> String {
    let path = path.display();
    match lang {
        Lang::Es => {
            format!("{path} es una carpeta del sistema. La app no organiza ahi por seguridad.")
        }
        Lang::En => {
            format!("{path} is a system folder. The app does not organize there, for safety.")
        }
    }
}

pub fn error_no_roots(lang: Lang) -> &'static str {
    match lang {
        Lang::Es => {
            "Todavía no hay ninguna carpeta elegida. Elige una primero y luego vuelve a intentar."
        }
        Lang::En => "No folder has been chosen yet. Choose one first and try again.",
    }
}

pub fn error_not_found(lang: Lang, path: &Path) -> String {
    let path = path.display();
    match lang {
        Lang::Es => format!("{path} ya no está en el disco, así que se omitió."),
        Lang::En => format!("{path} is no longer on disk, so it was skipped."),
    }
}

pub fn error_copy_verify(lang: Lang, path: &Path) -> String {
    let path = path.display();
    match lang {
        Lang::Es => format!(
            "La copia de {path} no coincidió en tamaño o contenido. El original quedó intacto y la copia parcial se borró. No se perdió nada."
        ),
        Lang::En => format!(
            "The copy of {path} did not match in size or content. The original was left untouched and the partial copy was removed. Nothing was lost."
        ),
    }
}

pub fn error_config_dir(lang: Lang) -> &'static str {
    match lang {
        Lang::Es => {
            "El sistema no dijo donde guardar los ajustes, así que la app no puede recordar nada todavía. Revisa las variables de entorno del usuario."
        }
        Lang::En => {
            "The system did not report where settings go, so the app cannot remember anything yet. Check the user environment variables."
        }
    }
}

pub fn error_watch(lang: Lang, detail: &str) -> String {
    match lang {
        Lang::Es => format!(
            "La vigilancia de carpetas no pudo arrancar. Los archivos siguen intactos y la organización manual sigue funcionando. Detalle: {detail}"
        ),
        Lang::En => format!(
            "Folder watching could not start. Your files are untouched and manual organizing still works. Detail: {detail}"
        ),
    }
}

pub fn error_trash(lang: Lang, path: &Path, detail: &str) -> String {
    let path = path.display();
    match lang {
        Lang::Es => format!(
            "No se pudo mover {path} a la papelera. El archivo sigue donde estaba. Detalle: {detail}"
        ),
        Lang::En => format!(
            "{path} could not be moved to the trash. The file is still where it was. Detail: {detail}"
        ),
    }
}

pub fn preview_summary(lang: Lang, files: usize, destinations: &str) -> String {
    match lang {
        Lang::Es => format!("Se organizarían {files} archivos en {destinations}."),
        Lang::En => format!("{files} files would be organized into {destinations}."),
    }
}

pub fn moved_notice(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es => format!(
            "Se organizaron {files} archivos. Puedes deshacerlo desde el historial o con Ctrl+Z."
        ),
        Lang::En => format!("Organized {files} files. Undo from history or with Ctrl+Z."),
    }
}

pub fn watching(lang: Lang, folders: usize) -> String {
    match (lang, folders) {
        (Lang::Es, 1) => "Vigilando 1 carpeta".to_string(),
        (Lang::Es, _) => format!("Vigilando {folders} carpetas"),
        (Lang::En, 1) => "Watching 1 folder".to_string(),
        (Lang::En, _) => format!("Watching {folders} folders"),
    }
}

pub fn active_folders(lang: Lang, folders: usize) -> String {
    match (lang, folders) {
        (Lang::Es, 1) => "1 carpeta activa".to_string(),
        (Lang::Es, _) => format!("{folders} carpetas activas"),
        (Lang::En, 1) => "1 active folder".to_string(),
        (Lang::En, _) => format!("{folders} active folders"),
    }
}

pub fn last_organized(lang: Lang, when: &str) -> String {
    match lang {
        Lang::Es => format!("Última organización: {when}"),
        Lang::En => format!("Last organized: {when}"),
    }
}

pub fn scanning(lang: Lang, done: usize, total: usize) -> String {
    match lang {
        Lang::Es => format!("Escaneando {done} de {total} archivos"),
        Lang::En => format!("Scanning {done} of {total} files"),
    }
}

pub fn csv_header(lang: Lang) -> &'static str {
    match lang {
        Lang::Es => "lote,fecha,estado,operación,origen,destino,tamaño,grupo",
        Lang::En => "batch,date,state,operation,source,destination,size,group",
    }
}

pub fn bytes_size(lang: Lang, bytes: u64) -> String {
    let (value, unit) = if bytes >= 1_073_741_824 {
        (bytes as f64 / 1_073_741_824.0, "GB")
    } else if bytes >= 1_048_576 {
        (bytes as f64 / 1_048_576.0, "MB")
    } else if bytes >= 1024 {
        (bytes as f64 / 1024.0, "KB")
    } else {
        (bytes as f64, "B")
    };
    let _ = lang;
    if unit == "B" {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {unit}")
    }
}

pub fn files_label(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es if files == 1 => "1 archivo".to_string(),
        Lang::Es => format!("{files} archivos"),
        Lang::En if files == 1 => "1 file".to_string(),
        Lang::En => format!("{files} files"),
    }
}

pub fn files_ready(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es if files == 1 => "1 archivo listo".to_string(),
        Lang::Es => format!("{files} archivos listos"),
        Lang::En if files == 1 => "1 file ready".to_string(),
        Lang::En => format!("{files} files ready"),
    }
}

pub fn trashed_label(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es if files == 1 => "1 a la papelera".to_string(),
        Lang::Es => format!("{files} a la papelera"),
        Lang::En if files == 1 => "1 to trash".to_string(),
        Lang::En => format!("{files} to trash"),
    }
}

pub fn review_label(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es if files == 1 => "1 para revisar".to_string(),
        Lang::Es => format!("{files} para revisar"),
        Lang::En if files == 1 => "1 needs review".to_string(),
        Lang::En => format!("{files} need review"),
    }
}

pub fn wizard_step(lang: Lang, step: u8, total: u8) -> String {
    match lang {
        Lang::Es => format!("Paso {step} de {total}"),
        Lang::En => format!("Step {step} of {total}"),
    }
}

pub fn confirm_trash_body(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es => format!(
            "Se moverían {files} archivos a la Papelera del sistema. Podrás recuperarlos desde la Papelera y el lote queda registrado en el historial."
        ),
        Lang::En => format!(
            "{files} files would move to the system Trash. You can restore them from the Trash and the batch is recorded in history."
        ),
    }
}

pub fn confirm_move_body(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es => format!(
            "Se moverían {files} archivos. Nada se borra: los originales se conservan y el lote se puede deshacer."
        ),
        Lang::En => format!(
            "{files} files would move. Nothing is deleted: originals are kept and the batch can be undone."
        ),
    }
}

pub fn plan_ready(lang: Lang, files: usize, destinations: &str) -> String {
    match lang {
        Lang::Es => {
            if destinations.is_empty() {
                format!("{files} archivos listos para organizar")
            } else {
                format!("{files} archivos listos para organizar en {destinations}")
            }
        }
        Lang::En => {
            if destinations.is_empty() {
                format!("{files} files ready to organize")
            } else {
                format!("{files} files ready to organize into {destinations}")
            }
        }
    }
}

pub fn undo_confirm_body(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es => format!(
            "Se devolverían {files} archivos a su sitio original. Los que ya no estén o cuyo hueco esté ocupado se quedan como están."
        ),
        Lang::En => format!(
            "{files} files would return to their original place. Those that are gone or whose spot is taken stay as they are."
        ),
    }
}

pub fn skipped_note(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es => format!(
            "{files} archivos no se tocan por ser recientes, estar ocultos o ya estar en su sitio."
        ),
        Lang::En => format!(
            "{files} files are left alone because they are recent, hidden or already in place."
        ),
    }
}

pub fn undo_done(lang: Lang, files: usize) -> String {
    match lang {
        Lang::Es => format!(
            "Se deshizo el lote: {files} archivos volvieron a su sitio o se retiró la copia."
        ),
        Lang::En => {
            format!("Batch undone: {files} files returned to their place or the copy was removed.")
        }
    }
}

pub fn undo_partial(lang: Lang, files: usize, skipped: usize) -> String {
    match lang {
        Lang::Es => format!(
            "Se devolvieron {files} archivos. {skipped} no se pudieron mover porque su sitio original está ocupado o el archivo ya no existe."
        ),
        Lang::En => format!(
            "Returned {files} files. {skipped} could not move because their original place is taken or the file is gone."
        ),
    }
}

pub fn export_done(lang: Lang, path: &Path) -> String {
    let path = path.display();
    match lang {
        Lang::Es => format!("Historial exportado a {path}"),
        Lang::En => format!("History exported to {path}"),
    }
}

pub fn cli_moved(lang: Lang, moved: usize, trashed: usize, bytes: u64) -> String {
    let size = bytes_size(lang, bytes);
    match lang {
        Lang::Es => {
            if trashed > 0 {
                format!(
                    "Listo: {moved} archivos movidos y {trashed} a la papelera ({size}). Puedes deshacer los movimientos con --undo-last."
                )
            } else {
                format!(
                    "Listo: {moved} archivos movidos ({size}). Puedes deshacerlo con --undo-last."
                )
            }
        }
        Lang::En => {
            if trashed > 0 {
                format!(
                    "Done: {moved} files moved and {trashed} sent to trash ({size}). Undo the moves with --undo-last."
                )
            } else {
                format!("Done: {moved} files moved ({size}). Undo with --undo-last.")
            }
        }
    }
}

pub fn cli_watching_start(lang: Lang, folders: usize) -> String {
    match lang {
        Lang::Es => format!(
            "Vigilando {folders} carpetas. Los archivos nuevos se organizan solos. Ctrl+C para salir."
        ),
        Lang::En => format!(
            "Watching {folders} folders. New files are organized automatically. Ctrl+C to quit."
        ),
    }
}

pub fn cli_help(lang: Lang) -> String {
    let name = t(lang, Key::AppName);
    match lang {
        Lang::Es => format!(
            "{name} {version}\n\nUso:\n  smart-organizer-cli [opciones]\n\nOpciones:\n  --organize <ruta>  Organiza esa carpeta con los ajustes guardados.\n  --dry-run          Muestra que se movería, sin tocar nada.\n  --undo-last        Deshace el último lote del diario.\n  --watch            Queda vigilando y organiza cuando llegan archivos.\n  -h, --help         Muestra esta ayuda.\n  -V, --version      Muestra la versión.\n",
            version = env!("CARGO_PKG_VERSION"),
        ),
        Lang::En => format!(
            "{name} {version}\n\nUsage:\n  smart-organizer-cli [options]\n\nOptions:\n  --organize <path>  Organize that folder with the saved settings.\n  --dry-run          Show what would move, touching nothing.\n  --undo-last        Undo the last batch in the journal.\n  --watch            Stay watching and organize as files arrive.\n  -h, --help         Show this help.\n  -V, --version      Show the version.\n",
            version = env!("CARGO_PKG_VERSION"),
        ),
    }
}

pub fn unknown_flag(lang: Lang, flag: &str) -> String {
    match lang {
        Lang::Es => format!("La opción {flag} no existe."),
        Lang::En => format!("The option {flag} does not exist."),
    }
}

pub fn missing_value(lang: Lang, flag: &str) -> String {
    match lang {
        Lang::Es => format!("La opción {flag} necesita una ruta."),
        Lang::En => format!("The option {flag} needs a path."),
    }
}

pub fn profile_changes(lang: Lang, groups: usize, rules: usize, skipped: usize) -> String {
    match lang {
        Lang::Es => {
            if groups == 0 && rules == 0 {
                t(lang, Key::ProfileNothingNew).to_string()
            } else {
                let mut text = format!("Se añadirán {groups} grupos y {rules} reglas.");
                if skipped > 0 {
                    text.push(' ');
                    text.push_str(t(lang, Key::ProfileSkipped));
                }
                text
            }
        }
        Lang::En => {
            if groups == 0 && rules == 0 {
                t(lang, Key::ProfileNothingNew).to_string()
            } else {
                let mut text = format!("Will add {groups} groups and {rules} rules.");
                if skipped > 0 {
                    text.push(' ');
                    text.push_str(t(lang, Key::ProfileSkipped));
                }
                text
            }
        }
    }
}

pub fn drop_added(lang: Lang, folders: usize) -> String {
    match lang {
        Lang::Es => {
            format!("{folders} carpetas añadidas. Revisa la vista previa antes de mover nada.")
        }
        Lang::En => format!("{folders} folders added. Check the preview before moving anything."),
    }
}

pub fn drop_rejected(lang: Lang, rejected: usize) -> String {
    match lang {
        Lang::Es => {
            format!("{rejected} rutas no se añadieron: no son carpetas válidas o son del sistema.")
        }
        Lang::En => format!(
            "{rejected} paths were not added: they are not valid folders or belong to the system."
        ),
    }
}

pub fn drop_none(lang: Lang) -> String {
    match lang {
        Lang::Es => "No se pudo añadir ninguna de las rutas soltadas.".to_string(),
        Lang::En => "None of the dropped paths could be added.".to_string(),
    }
}

pub fn duplicates_found(lang: Lang, groups: usize, files: usize, bytes: u64) -> String {
    let size = bytes_size(lang, bytes);
    match lang {
        Lang::Es => format!("{groups} grupos · {files} archivos · {size} por liberar"),
        Lang::En => format!("{groups} groups · {files} files · {size} to free up"),
    }
}

pub fn duplicates_confirm(lang: Lang, files: usize, bytes: u64) -> String {
    let size = bytes_size(lang, bytes);
    match lang {
        Lang::Es => format!(
            "Se enviarán {files} archivos a la papelera y se liberarán {size}. Se conserva una copia de cada grupo."
        ),
        Lang::En => format!(
            "{files} files will go to the Recycle Bin and {size} will be freed. One copy from each group is kept."
        ),
    }
}

pub fn duplicates_done(lang: Lang, files: usize, bytes: u64, failed: usize) -> String {
    let size = bytes_size(lang, bytes);
    match lang {
        Lang::Es => {
            if failed == 0 {
                format!("{files} archivos enviados a la papelera ({size}).")
            } else {
                format!(
                    "{files} archivos enviados a la papelera ({size}); {failed} no se pudieron borrar."
                )
            }
        }
        Lang::En => {
            if failed == 0 {
                format!("{files} files sent to the Recycle Bin ({size}).")
            } else {
                format!(
                    "{files} files sent to the Recycle Bin ({size}); {failed} could not be deleted."
                )
            }
        }
    }
}

pub fn duplicates_progress(lang: Lang, done: usize, total: usize) -> String {
    match lang {
        Lang::Es => format!("{done} de {total} archivos comparados"),
        Lang::En => format!("{done} of {total} files compared"),
    }
}
