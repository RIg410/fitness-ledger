
export class UserView {
    id: string;
    tg_id: number;
    name: UserName;
    rights: Rights;
    phone?: string;
    is_active: boolean;
    freeze?: Freeze;
    subscriptions: UserSubscription[];
    freeze_days: number;
    created_at: Date;
    employee?: Employee;
    family: Family;
    birthday?: Birthday;

    constructor(data: any) {
        this.id = data.id;
        this.tg_id = data.tg_id;
        this.name = data.name;
        this.rights = data.rights;
        this.phone = data.phone;
        this.is_active = data.is_active;
        this.freeze = data.freeze;
        this.subscriptions = data.subscriptions;
        this.freeze_days = data.freeze_days;
        this.created_at = new Date(data.created_at);
        this.employee = data.employee;
        this.family = data.family;
        this.birthday = data.birthday;
    }
}

export class UserName {
    tg_user_name?: string;
    first_name: string;
    last_name?: string;

    constructor(data: any) {
        this.tg_user_name = data.tg_user_name;
        this.first_name = data.first_name;
        this.last_name = data.last_name;
    }
}

export class Rights {
    full: boolean;
    rights: Rule[];

    constructor(data: any) {
        this.full = data.full;
        this.rights = data.rights;
    }
}

export enum Rule {
    ViewProfile,

    // User menu
    ViewUsers,
    EditUserRights,
    BlockUser,
    EditUserInfo,
    EditUserSubscription,
    FreezeUsers,
    ChangeBalance,
    EditMarketingInfo,
    EditFamily,
    ViewFamily,

    // Training
    EditTraining,
    CancelTraining,
    CreateTraining,
    EditSchedule,
    EditTrainingClientsList,
    SetKeepOpen,
    SetFree,

    // Subscription
    CreateSubscription,
    EditSubscription,
    SellSubscription,
    FreeSell,

    //Finance
    ViewFinance,
    MakePayment,
    MakeDeposit,
    FinanceHistoricalDate,
    DeleteHistory,

    //Employees
    ViewEmployees,
    EditEmployee,

    //Logs
    ViewLogs,

    //Couching
    CreateCouch,
    EditCouch,
    ViewCouchRates,

    //statistics
    ViewStatistics,

    System,

    ViewRewards,
    RecalculateRewards,

    ViewMarketingInfo,
    CreateRequest,
    RequestsHistory,

    ReceiveNotificationsAboutSubscriptions,

    // experimental
    MiniApp,
    BuySubscription,

    // program
    ViewHiddenPrograms,
    HistoryViewer,
}

export class Freeze {
    freeze_start: Date;
    freeze_end: Date;

    constructor(data: any) {
        this.freeze_start = new Date(data.freeze_start);
        this.freeze_end = new Date(data.freeze_end);
    }
}

export class UserSubscription {
    id: string;
    subscription_id: string;
    name: string;
    items: number;
    active?: Active;
    is_group: boolean;
    balance: number;
    locked_balance: number;
    unlimited: boolean;

    constructor(data: any) {
        this.id = data.id;
        this.subscription_id = data.subscription_id;
        this.name = data.name;
        this.items = data.items;
        this.balance = data.balance;
        this.locked_balance = data.locked_balance;
        this.unlimited = data.unlimited;
    }
}

export class Active {
    start: Date;
    end: Date;

    constructor(data: any) {
        this.start = new Date(data.start);
        this.end = new Date(data.end);
    }
}


export class Family {
    payer?: UserView;
    children: UserView[];

    constructor(data: any) {
        this.payer = data.payer;
        this.children = data.children;
    }
}

export class Employee {
    role: EmployeeRole;
    description: string;
    reward: number;
    rates: Rate[];
}

export class Birthday {
    day: number;
    month: number;
    year: number;

    constructor(data: any) {
        this.day = data.day;
        this.month = data.month;
        this.year = data.year;
    }
}

export enum EmployeeRole {
    Coach,
    Manager,
    Admin,
}

export class Rate {
    fix?: Fix;
    fix_by_training?: FixByTraining;
    training_percent?: TrainingPercent;
}

export class Fix {
    amount: number;
    last_payment_date: Date;
    next_payment_date: Date;
    interval: number;

    constructor(data: any) {
        this.amount = data.amount;
        this.last_payment_date = new Date(data.last_payment_date);
        this.next_payment_date = new Date(data.next_payment_date);
        this.interval = data.interval;
    }
}

export class FixByTraining {
    amount: number;

    constructor(data: any) {
        this.amount = data.amount;
    }
}

export class TrainingPercent {
    percent: number;
    min_reward?: number;

    constructor(data: any) {
        this.percent = data.percent;
        this.min_reward = data.min_reward;
    }
}