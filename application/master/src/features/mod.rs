//! Application Master feature modules.

pub mod al;
pub mod dashboard;
pub mod db;
pub mod su;
pub mod localization;
pub mod referential_integrity;

pub use al::alpg000::{
    Alpg000CommandExecutor, Alpg000Commands, Alpg000Error, AuditLogDto, AuditLogFilterQuery,
    AuditLogPageDto, AuditLogSummaryDto, AuditStatusDto, ClientActor, ClientEventCommand,
    normalize_client_events, normalize_search,
};
pub use db::dbpg000::{
    CompanyDto, CompanyFilterQuery, CompanyTypeLookupDto, DbDbpg000Queries, Dbpg000CommandExecutor,
    Dbpg000Commands, Dbpg000Error, Dbpg000Queries, OrganizationDto, OrganizationSummaryDto,
};
pub use db::dbpg002::{
    BranchDto, BranchFilterQuery, BranchTypeLookupDto, CompanyOptionDto, CreateBranchCommand,
    CreateDepartmentCommand, DbDbpg002Queries, Dbpg002CommandExecutor, Dbpg002Commands,
    Dbpg002Error, Dbpg002Queries, Dbpg002SummaryDto, DepartmentDto, DepartmentFilterQuery,
    UpdateBranchCommand, UpdateDepartmentCommand,
};
pub use db::dbpg003::{
    BranchOptionDto, CreateEmployeeCommand, DbDbpg003Queries, Dbpg003CommandExecutor,
    Dbpg003Commands, Dbpg003Error, Dbpg003Queries, Dbpg003SummaryDto, DepartmentOptionDto,
    EmployeeDto, EmployeeFilterQuery, EmployeePageDto, PrefixOptionDto, UpdateEmployeeCommand,
};
pub use db::dbpg004::{
    CreateLookupCommand, DbDbpg004Queries, Dbpg004CommandExecutor, Dbpg004Commands, Dbpg004Error,
    Dbpg004Queries, Dbpg004SummaryDto, LookupCategoryDto, LookupDto, LookupFilterQuery,
    LookupPageDto, UpdateLookupCommand,
};
pub use db::dbpg005::{
    CreateDistrictCommand, CreatePostalCodeCommand, CreateProvinceCommand,
    CreateSubDistrictCommand, DbDbpg005Queries, Dbpg005CommandExecutor, Dbpg005Commands,
    Dbpg005Error, Dbpg005Queries, Dbpg005SummaryDto, GeoFilterQuery, GeoNodeDto, GeoPageDto,
    PostalCodeDto, PostalFilterQuery, PostalPageDto, UpdateGeoNodeCommand, UpdatePostalCodeCommand,
};
pub use db::dbpg006::{
    CreateLabelCommand, DbDbpg006Queries, Dbpg006CommandExecutor, Dbpg006Commands, Dbpg006Error,
    Dbpg006Queries, LabelCategoryDto, LabelEntryDto, LabelFilterQuery, LabelPageDto,
    LabelValueDto, LabelValueInput, LanguageOptionDto, UpdateLabelCommand,
    normalize_label_category, normalize_label_key,
};
pub use db::dbpg001::{
    CreateCompanyCommand, CreateOrganizationCommand, DbDbpg001Queries, Dbpg001CommandExecutor,
    Dbpg001Commands, Dbpg001Error, Dbpg001Queries, UpdateCompanyCommand, UpdateOrganizationCommand,
};

pub use dashboard::{
    commands::{
        DashboardCommandExecutor, DashboardCommands, GetDashboardCommand,
        GetDashboardCommandHandler,
    },
    errors::DashboardError,
    models::{DashboardKpis, DashboardSummary, ModuleItem, ProgramItem},
    queries::{DashboardQueries, DbDashboardQueries, GetDashboardStatsQuery},
};
pub use su::access_control::{
    AccessControl, AccessError, Actor, DbAccessControl, PermissionAction,
};
pub use su::supg001::{
    CreateModuleCommand, CreateSystemCommand, DbSupg001Queries, ModuleDto, ModuleFilterQuery,
    Supg001CommandExecutor, Supg001Commands, Supg001Error, Supg001Queries, SystemDto,
    SystemFilterQuery, UpdateUnitCommand,
};
pub use su::supg002::{
    CreateProgramCommand, DbSupg002Queries, MenuNodeDto, ProgramDto, ProgramFilterQuery,
    ProgramOptionsDto, ProgramPageDto, Supg002CommandExecutor, Supg002Commands, Supg002Error,
    Supg002Queries, UpdateProgramCommand,
};
pub use su::supg003::{
    CreatePermissionGroupCommand, DbSupg003Queries, GrantDto, GroupFilterQuery, MatrixAction,
    MatrixRowDto, PermissionGroupDto, PermissionMatrixDto, SaveMatrixCommand,
    Supg003CommandExecutor, Supg003Commands, Supg003Error, Supg003Queries,
    UpdatePermissionGroupCommand,
};
pub use su::supg004::{
    CreateUserCommand, CreateUserResultDto, DbSupg004Queries, EmployeeOptionDto,
    EmployeeOptionsQuery, GroupOptionDto, RecoveryLinkDto, Supg004CommandExecutor, Supg004Commands,
    Supg004Error, Supg004Queries, UpdateUserCommand, UserAccountDto, UserFilterQuery, UserPageDto,
};
pub use su::supg005::{
    commands::{Supg005CommandExecutor, Supg005Commands},
    errors::Supg005Error,
    models::{ActiveDeviceDto as Supg005ActiveDeviceDto, RevokeResultDto, UserSessionOverviewDto},
    queries::{DbSupg005Queries, Supg005Queries},
};
pub use localization::{
    commands::{
        GetI18nCommand, GetI18nCommandHandler, I18nCommandExecutor, LocalizationCommandService,
        LocalizationCommands,
    },
    errors::LocalizationError,
    models::I18nDictionary,
    queries::{
        DbLabelQueries, GetI18nDictionaryHandler, GetI18nDictionaryQuery, LocalizationQueries,
        LocalizationQueryService,
    },
};
