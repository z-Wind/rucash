--
-- PostgreSQL database dump
--

-- Dumped from database version 12.7 (Ubuntu 12.7-0ubuntu0.20.04.1)
-- Dumped by pg_dump version 12.7 (Ubuntu 12.7-0ubuntu0.20.04.1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: accounts; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.accounts (
    guid character varying(32) NOT NULL,
    name character varying(2048) NOT NULL,
    account_type character varying(2048) NOT NULL,
    commodity_guid character varying(32),
    commodity_scu integer NOT NULL,
    non_std_scu integer NOT NULL,
    parent_guid character varying(32),
    code character varying(2048),
    description character varying(2048),
    hidden integer,
    placeholder integer
);


ALTER TABLE public.accounts OWNER TO postgres;

--
-- Name: billterms; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.billterms (
    guid character varying(32) NOT NULL,
    name character varying(2048) NOT NULL,
    description character varying(2048) NOT NULL,
    refcount integer NOT NULL,
    invisible integer NOT NULL,
    parent character varying(32),
    type character varying(2048) NOT NULL,
    duedays integer,
    discountdays integer,
    discount_num bigint,
    discount_denom bigint,
    cutoff integer
);


ALTER TABLE public.billterms OWNER TO postgres;

--
-- Name: books; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.books (
    guid character varying(32) NOT NULL,
    root_account_guid character varying(32) NOT NULL,
    root_template_guid character varying(32) NOT NULL
);


ALTER TABLE public.books OWNER TO postgres;

--
-- Name: budget_amounts; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.budget_amounts (
    id integer NOT NULL,
    budget_guid character varying(32) NOT NULL,
    account_guid character varying(32) NOT NULL,
    period_num integer NOT NULL,
    amount_num bigint NOT NULL,
    amount_denom bigint NOT NULL
);


ALTER TABLE public.budget_amounts OWNER TO postgres;

--
-- Name: budget_amounts_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.budget_amounts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.budget_amounts_id_seq OWNER TO postgres;

--
-- Name: budget_amounts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.budget_amounts_id_seq OWNED BY public.budget_amounts.id;


--
-- Name: budgets; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.budgets (
    guid character varying(32) NOT NULL,
    name character varying(2048) NOT NULL,
    description character varying(2048),
    num_periods integer NOT NULL
);


ALTER TABLE public.budgets OWNER TO postgres;

--
-- Name: commodities; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.commodities (
    guid character varying(32) NOT NULL,
    namespace character varying(2048) NOT NULL,
    mnemonic character varying(2048) NOT NULL,
    fullname character varying(2048),
    cusip character varying(2048),
    fraction integer NOT NULL,
    quote_flag integer NOT NULL,
    quote_source character varying(2048),
    quote_tz character varying(2048)
);


ALTER TABLE public.commodities OWNER TO postgres;

--
-- Name: customers; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.customers (
    guid character varying(32) NOT NULL,
    name character varying(2048) NOT NULL,
    id character varying(2048) NOT NULL,
    notes character varying(2048) NOT NULL,
    active integer NOT NULL,
    discount_num bigint NOT NULL,
    discount_denom bigint NOT NULL,
    credit_num bigint NOT NULL,
    credit_denom bigint NOT NULL,
    currency character varying(32) NOT NULL,
    tax_override integer NOT NULL,
    addr_name character varying(1024),
    addr_addr1 character varying(1024),
    addr_addr2 character varying(1024),
    addr_addr3 character varying(1024),
    addr_addr4 character varying(1024),
    addr_phone character varying(128),
    addr_fax character varying(128),
    addr_email character varying(256),
    shipaddr_name character varying(1024),
    shipaddr_addr1 character varying(1024),
    shipaddr_addr2 character varying(1024),
    shipaddr_addr3 character varying(1024),
    shipaddr_addr4 character varying(1024),
    shipaddr_phone character varying(128),
    shipaddr_fax character varying(128),
    shipaddr_email character varying(256),
    terms character varying(32),
    tax_included integer,
    taxtable character varying(32)
);


ALTER TABLE public.customers OWNER TO postgres;

--
-- Name: employees; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.employees (
    guid character varying(32) NOT NULL,
    username character varying(2048) NOT NULL,
    id character varying(2048) NOT NULL,
    language character varying(2048) NOT NULL,
    acl character varying(2048) NOT NULL,
    active integer NOT NULL,
    currency character varying(32) NOT NULL,
    ccard_guid character varying(32),
    workday_num bigint NOT NULL,
    workday_denom bigint NOT NULL,
    rate_num bigint NOT NULL,
    rate_denom bigint NOT NULL,
    addr_name character varying(1024),
    addr_addr1 character varying(1024),
    addr_addr2 character varying(1024),
    addr_addr3 character varying(1024),
    addr_addr4 character varying(1024),
    addr_phone character varying(128),
    addr_fax character varying(128),
    addr_email character varying(256)
);


ALTER TABLE public.employees OWNER TO postgres;

--
-- Name: entries; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.entries (
    guid character varying(32) NOT NULL,
    date timestamp without time zone NOT NULL,
    date_entered timestamp without time zone,
    description character varying(2048),
    action character varying(2048),
    notes character varying(2048),
    quantity_num bigint,
    quantity_denom bigint,
    i_acct character varying(32),
    i_price_num bigint,
    i_price_denom bigint,
    i_discount_num bigint,
    i_discount_denom bigint,
    invoice character varying(32),
    i_disc_type character varying(2048),
    i_disc_how character varying(2048),
    i_taxable integer,
    i_taxincluded integer,
    i_taxtable character varying(32),
    b_acct character varying(32),
    b_price_num bigint,
    b_price_denom bigint,
    bill character varying(32),
    b_taxable integer,
    b_taxincluded integer,
    b_taxtable character varying(32),
    b_paytype integer,
    billable integer,
    billto_type integer,
    billto_guid character varying(32),
    order_guid character varying(32)
);


ALTER TABLE public.entries OWNER TO postgres;

--
-- Name: gnclock; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.gnclock (
    hostname character varying(255),
    pid integer
);


ALTER TABLE public.gnclock OWNER TO postgres;

--
-- Name: invoices; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.invoices (
    guid character varying(32) NOT NULL,
    id character varying(2048) NOT NULL,
    date_opened timestamp without time zone,
    date_posted timestamp without time zone,
    notes character varying(2048) NOT NULL,
    active integer NOT NULL,
    currency character varying(32) NOT NULL,
    owner_type integer,
    owner_guid character varying(32),
    terms character varying(32),
    billing_id character varying(2048),
    post_txn character varying(32),
    post_lot character varying(32),
    post_acc character varying(32),
    billto_type integer,
    billto_guid character varying(32),
    charge_amt_num bigint,
    charge_amt_denom bigint
);


ALTER TABLE public.invoices OWNER TO postgres;

--
-- Name: jobs; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.jobs (
    guid character varying(32) NOT NULL,
    id character varying(2048) NOT NULL,
    name character varying(2048) NOT NULL,
    reference character varying(2048) NOT NULL,
    active integer NOT NULL,
    owner_type integer,
    owner_guid character varying(32)
);


ALTER TABLE public.jobs OWNER TO postgres;

--
-- Name: lots; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.lots (
    guid character varying(32) NOT NULL,
    account_guid character varying(32),
    is_closed integer NOT NULL
);


ALTER TABLE public.lots OWNER TO postgres;

--
-- Name: orders; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.orders (
    guid character varying(32) NOT NULL,
    id character varying(2048) NOT NULL,
    notes character varying(2048) NOT NULL,
    reference character varying(2048) NOT NULL,
    active integer NOT NULL,
    date_opened timestamp without time zone NOT NULL,
    date_closed timestamp without time zone NOT NULL,
    owner_type integer NOT NULL,
    owner_guid character varying(32) NOT NULL
);


ALTER TABLE public.orders OWNER TO postgres;

--
-- Name: prices; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.prices (
    guid character varying(32) NOT NULL,
    commodity_guid character varying(32) NOT NULL,
    currency_guid character varying(32) NOT NULL,
    date timestamp without time zone NOT NULL,
    source character varying(2048),
    type character varying(2048),
    value_num bigint NOT NULL,
    value_denom bigint NOT NULL
);


ALTER TABLE public.prices OWNER TO postgres;

--
-- Name: recurrences; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.recurrences (
    id integer NOT NULL,
    obj_guid character varying(32) NOT NULL,
    recurrence_mult integer NOT NULL,
    recurrence_period_type character varying(2048) NOT NULL,
    recurrence_period_start date NOT NULL,
    recurrence_weekend_adjust character varying(2048) NOT NULL
);


ALTER TABLE public.recurrences OWNER TO postgres;

--
-- Name: recurrences_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.recurrences_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.recurrences_id_seq OWNER TO postgres;

--
-- Name: recurrences_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.recurrences_id_seq OWNED BY public.recurrences.id;


--
-- Name: schedxactions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.schedxactions (
    guid character varying(32) NOT NULL,
    name character varying(2048),
    enabled integer NOT NULL,
    start_date date,
    end_date date,
    last_occur date,
    num_occur integer NOT NULL,
    rem_occur integer NOT NULL,
    auto_create integer NOT NULL,
    auto_notify integer NOT NULL,
    adv_creation integer NOT NULL,
    adv_notify integer NOT NULL,
    instance_count integer NOT NULL,
    template_act_guid character varying(32) NOT NULL
);


ALTER TABLE public.schedxactions OWNER TO postgres;

--
-- Name: slots; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.slots (
    id integer NOT NULL,
    obj_guid character varying(32) NOT NULL,
    name character varying(4096) NOT NULL,
    slot_type integer NOT NULL,
    int64_val bigint,
    string_val character varying(4096),
    double_val double precision,
    timespec_val timestamp without time zone,
    guid_val character varying(32),
    numeric_val_num bigint,
    numeric_val_denom bigint,
    gdate_val date
);


ALTER TABLE public.slots OWNER TO postgres;

--
-- Name: slots_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.slots_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.slots_id_seq OWNER TO postgres;

--
-- Name: slots_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.slots_id_seq OWNED BY public.slots.id;


--
-- Name: splits; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.splits (
    guid character varying(32) NOT NULL,
    tx_guid character varying(32) NOT NULL,
    account_guid character varying(32) NOT NULL,
    memo character varying(2048) NOT NULL,
    action character varying(2048) NOT NULL,
    reconcile_state character varying(1) NOT NULL,
    reconcile_date timestamp without time zone,
    value_num bigint NOT NULL,
    value_denom bigint NOT NULL,
    quantity_num bigint NOT NULL,
    quantity_denom bigint NOT NULL,
    lot_guid character varying(32)
);


ALTER TABLE public.splits OWNER TO postgres;

--
-- Name: taxtable_entries; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.taxtable_entries (
    id integer NOT NULL,
    taxtable character varying(32) NOT NULL,
    account character varying(32) NOT NULL,
    amount_num bigint NOT NULL,
    amount_denom bigint NOT NULL,
    type integer NOT NULL
);


ALTER TABLE public.taxtable_entries OWNER TO postgres;

--
-- Name: taxtable_entries_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.taxtable_entries_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.taxtable_entries_id_seq OWNER TO postgres;

--
-- Name: taxtable_entries_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.taxtable_entries_id_seq OWNED BY public.taxtable_entries.id;


--
-- Name: taxtables; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.taxtables (
    guid character varying(32) NOT NULL,
    name character varying(50) NOT NULL,
    refcount bigint NOT NULL,
    invisible integer NOT NULL,
    parent character varying(32)
);


ALTER TABLE public.taxtables OWNER TO postgres;

--
-- Name: transactions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.transactions (
    guid character varying(32) NOT NULL,
    currency_guid character varying(32) NOT NULL,
    num character varying(2048) NOT NULL,
    post_date timestamp without time zone,
    enter_date timestamp without time zone,
    description character varying(2048)
);


ALTER TABLE public.transactions OWNER TO postgres;

--
-- Name: vendors; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.vendors (
    guid character varying(32) NOT NULL,
    name character varying(2048) NOT NULL,
    id character varying(2048) NOT NULL,
    notes character varying(2048) NOT NULL,
    currency character varying(32) NOT NULL,
    active integer NOT NULL,
    tax_override integer NOT NULL,
    addr_name character varying(1024),
    addr_addr1 character varying(1024),
    addr_addr2 character varying(1024),
    addr_addr3 character varying(1024),
    addr_addr4 character varying(1024),
    addr_phone character varying(128),
    addr_fax character varying(128),
    addr_email character varying(256),
    terms character varying(32),
    tax_inc character varying(2048),
    tax_table character varying(32)
);


ALTER TABLE public.vendors OWNER TO postgres;

--
-- Name: versions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.versions (
    table_name character varying(50) NOT NULL,
    table_version integer NOT NULL
);


ALTER TABLE public.versions OWNER TO postgres;

--
-- Name: budget_amounts id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.budget_amounts ALTER COLUMN id SET DEFAULT nextval('public.budget_amounts_id_seq'::regclass);


--
-- Name: recurrences id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.recurrences ALTER COLUMN id SET DEFAULT nextval('public.recurrences_id_seq'::regclass);


--
-- Name: slots id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.slots ALTER COLUMN id SET DEFAULT nextval('public.slots_id_seq'::regclass);


--
-- Name: taxtable_entries id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.taxtable_entries ALTER COLUMN id SET DEFAULT nextval('public.taxtable_entries_id_seq'::regclass);


--
-- Data for Name: accounts; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.accounts (guid, name, account_type, commodity_guid, commodity_scu, non_std_scu, parent_guid, code, description, hidden, placeholder) FROM stdin;
00622dda21937b29e494179de5013f82	Root Account	ROOT	\N	0	0	\N			0	0
fcd795021c976ba75621ec39e75f6214	Asset	ASSET	346629655191dcf59a7e2c2a85b70f69	100	0	00622dda21937b29e494179de5013f82			0	1
3bc319753945b6dba3e1928abed49e35	Current	ASSET	346629655191dcf59a7e2c2a85b70f69	100	0	fcd795021c976ba75621ec39e75f6214			0	1
93fc043c3062aaa1297b30e543d2cd0d	Cash	ASSET	346629655191dcf59a7e2c2a85b70f69	100	0	3bc319753945b6dba3e1928abed49e35			0	0
a1dd9e8118bff87db289482bceebfea9	Savings	ASSET	346629655191dcf59a7e2c2a85b70f69	100	0	3bc319753945b6dba3e1928abed49e35			0	0
adc619f0ac7fa27d5768bfd73ecbc01e	Checking	ASSET	346629655191dcf59a7e2c2a85b70f69	100	0	3bc319753945b6dba3e1928abed49e35			0	0
5b5100d58bcc030d2f7828d897fda62e	Fixed	ASSET	346629655191dcf59a7e2c2a85b70f69	100	0	fcd795021c976ba75621ec39e75f6214			0	0
a3dc764fea53b709af7fcead6470da43	House	ASSET	346629655191dcf59a7e2c2a85b70f69	100	0	5b5100d58bcc030d2f7828d897fda62e			0	0
01f6b1417935528fdc97ac2e130a150c	Broker	ASSET	5f586908098232e67edb1371408bfaa8	100	0	fcd795021c976ba75621ec39e75f6214			0	1
1c089803052e85f5c6d8e786057dbaee	Foo stock	STOCK	069410ffec45a41a218bba474093d466	10000	0	01f6b1417935528fdc97ac2e130a150c			0	0
96ed7a45459fb5fe570e48fcd46f05d0	Liability	LIABILITY	346629655191dcf59a7e2c2a85b70f69	100	0	00622dda21937b29e494179de5013f82			0	0
6bbc8f20544452cac1637fb9a9b851bb	Income	INCOME	346629655191dcf59a7e2c2a85b70f69	100	0	00622dda21937b29e494179de5013f82			0	0
af88d386d44b14acf244362b85ccaf4c	Expense	EXPENSE	346629655191dcf59a7e2c2a85b70f69	100	0	00622dda21937b29e494179de5013f82			0	0
8056f425ecdc352cf6039e3a0d0d1e6c	Equity	EQUITY	346629655191dcf59a7e2c2a85b70f69	100	0	00622dda21937b29e494179de5013f82			0	0
1305127e63737f8c39afb49b5bbeca7a	Opening Balances - EUR	EQUITY	346629655191dcf59a7e2c2a85b70f69	100	0	8056f425ecdc352cf6039e3a0d0d1e6c			0	0
bb43218d7d95b1fe062a731d1aa7e9e2	Mouvements	TRADING	069410ffec45a41a218bba474093d466	10000	0	00622dda21937b29e494179de5013f82			0	1
f2b78b76f6093ba04204d972a7ce42d6	NASDAQ	TRADING	069410ffec45a41a218bba474093d466	10000	0	bb43218d7d95b1fe062a731d1aa7e9e2			0	1
0ccab772d0d16a3e1eaf42cd53f891e5	FOO	TRADING	069410ffec45a41a218bba474093d466	10000	0	f2b78b76f6093ba04204d972a7ce42d6			0	0
e97809296efb670cff4f92138bd69ec8	CURRENCY	TRADING	346629655191dcf59a7e2c2a85b70f69	100	0	bb43218d7d95b1fe062a731d1aa7e9e2			0	1
7894e9c3e955f5eaa9689d16ed775660	EUR5	TRADING	346629655191dcf59a7e2c2a85b70f69	100	0	e97809296efb670cff4f92138bd69ec8			0	0
f6c0cd00ec04169a44f170181882adab	Template Root	ROOT	\N	0	0	\N			0	0
\.


--
-- Data for Name: billterms; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.billterms (guid, name, description, refcount, invisible, parent, type, duedays, discountdays, discount_num, discount_denom, cutoff) FROM stdin;
\.


--
-- Data for Name: books; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.books (guid, root_account_guid, root_template_guid) FROM stdin;
7d4ef4044fd30f41d08914a8174c2f5b	00622dda21937b29e494179de5013f82	f6c0cd00ec04169a44f170181882adab
\.


--
-- Data for Name: budget_amounts; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.budget_amounts (id, budget_guid, account_guid, period_num, amount_num, amount_denom) FROM stdin;
\.


--
-- Data for Name: budgets; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.budgets (guid, name, description, num_periods) FROM stdin;
\.


--
-- Data for Name: commodities; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.commodities (guid, namespace, mnemonic, fullname, cusip, fraction, quote_flag, quote_source, quote_tz) FROM stdin;
346629655191dcf59a7e2c2a85b70f69	CURRENCY	EUR	Euro	978	100	1	currency	
5f586908098232e67edb1371408bfaa8	CURRENCY	AED	UAE Dirham	784	100	1	currency	
069410ffec45a41a218bba474093d466	NASDAQ	FOO	Foo Inc		10000	0	\N	
d821d6776fde9f7c2d01b67876406fd3	CURRENCY	ADF	Andorran Franc	950	100	0	currency	
1e5d65e2726a5d4595741cb204992991	CURRENCY	USD	US Dollar	840	100	0	currency	
\.


--
-- Data for Name: customers; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.customers (guid, name, id, notes, active, discount_num, discount_denom, credit_num, credit_denom, currency, tax_override, addr_name, addr_addr1, addr_addr2, addr_addr3, addr_addr4, addr_phone, addr_fax, addr_email, shipaddr_name, shipaddr_addr1, shipaddr_addr2, shipaddr_addr3, shipaddr_addr4, shipaddr_phone, shipaddr_fax, shipaddr_email, terms, tax_included, taxtable) FROM stdin;
\.


--
-- Data for Name: employees; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.employees (guid, username, id, language, acl, active, currency, ccard_guid, workday_num, workday_denom, rate_num, rate_denom, addr_name, addr_addr1, addr_addr2, addr_addr3, addr_addr4, addr_phone, addr_fax, addr_email) FROM stdin;
\.


--
-- Data for Name: entries; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.entries (guid, date, date_entered, description, action, notes, quantity_num, quantity_denom, i_acct, i_price_num, i_price_denom, i_discount_num, i_discount_denom, invoice, i_disc_type, i_disc_how, i_taxable, i_taxincluded, i_taxtable, b_acct, b_price_num, b_price_denom, bill, b_taxable, b_taxincluded, b_taxtable, b_paytype, billable, billto_type, billto_guid, order_guid) FROM stdin;
\.


--
-- Data for Name: gnclock; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.gnclock (hostname, pid) FROM stdin;
\.


--
-- Data for Name: invoices; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.invoices (guid, id, date_opened, date_posted, notes, active, currency, owner_type, owner_guid, terms, billing_id, post_txn, post_lot, post_acc, billto_type, billto_guid, charge_amt_num, charge_amt_denom) FROM stdin;
\.


--
-- Data for Name: jobs; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.jobs (guid, id, name, reference, active, owner_type, owner_guid) FROM stdin;
\.


--
-- Data for Name: lots; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.lots (guid, account_guid, is_closed) FROM stdin;
\.


--
-- Data for Name: orders; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.orders (guid, id, notes, reference, active, date_opened, date_closed, owner_type, owner_guid) FROM stdin;
\.


--
-- Data for Name: prices; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.prices (guid, commodity_guid, currency_guid, date, source, type, value_num, value_denom) FROM stdin;
0d6684f44fb018e882de76094ed9c433	d821d6776fde9f7c2d01b67876406fd3	5f586908098232e67edb1371408bfaa8	2018-02-20 23:00:00	user:price-editor	unknown	3	2
831ce95a146243a0b66895151538990a	5f586908098232e67edb1371408bfaa8	346629655191dcf59a7e2c2a85b70f69	2017-05-08 16:00:00	user:price-editor	last	1	2
c266e6ce9bdf8832bf88360df524669e	346629655191dcf59a7e2c2a85b70f69	5f586908098232e67edb1371408bfaa8	2018-02-20 23:00:00	user:price-editor	unknown	10	9
715ad31cb272437fc883d658f732a8eb	1e5d65e2726a5d4595741cb204992991	346629655191dcf59a7e2c2a85b70f69	2018-02-20 23:00:00	user:price-editor	unknown	7	5
86a98448a82e9012cad82ac677eb86df	069410ffec45a41a218bba474093d466	5f586908098232e67edb1371408bfaa8	2018-02-20 23:00:00	user:price-editor	unknown	9	10
\.


--
-- Data for Name: recurrences; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.recurrences (id, obj_guid, recurrence_mult, recurrence_period_type, recurrence_period_start, recurrence_weekend_adjust) FROM stdin;
\.


--
-- Data for Name: schedxactions; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.schedxactions (guid, name, enabled, start_date, end_date, last_occur, num_occur, rem_occur, auto_create, auto_notify, adv_creation, adv_notify, instance_count, template_act_guid) FROM stdin;
\.


--
-- Data for Name: slots; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.slots (id, obj_guid, name, slot_type, int64_val, string_val, double_val, timespec_val, guid_val, numeric_val_num, numeric_val_denom, gdate_val) FROM stdin;
1	7d4ef4044fd30f41d08914a8174c2f5b	features	9	0	\N	\N	1970-01-01 00:00:00	93187837476a4f008bad34cff13ae4b4	0	1	\N
2	93187837476a4f008bad34cff13ae4b4	features/ISO-8601 formatted date strings in SQLite3 databases.	4	0	Use ISO formatted date-time strings in SQLite3 databases (requires at least GnuCash 2.6.20)	\N	1970-01-01 00:00:00	\N	0	1	\N
3	7d4ef4044fd30f41d08914a8174c2f5b	options	9	0	\N	\N	1970-01-01 00:00:00	0d4e51bfe0374a3192cf7cd196176d13	0	1	\N
4	0d4e51bfe0374a3192cf7cd196176d13	options/Accounts	9	0	\N	\N	1970-01-01 00:00:00	464928d59a334579922f356594602a60	0	1	\N
5	464928d59a334579922f356594602a60	options/Accounts/Use Trading Accounts	4	0	t	\N	1970-01-01 00:00:00	\N	0	1	\N
6	0d4e51bfe0374a3192cf7cd196176d13	options/Budgeting	9	0	\N	\N	1970-01-01 00:00:00	4a6cd557a8e543bfab54d62c07a5dc0c	0	1	\N
7	7d4ef4044fd30f41d08914a8174c2f5b	remove-color-not-set-slots	4	0	true	\N	1970-01-01 00:00:00	\N	0	1	\N
8	fcd795021c976ba75621ec39e75f6214	placeholder	4	0	true	\N	1970-01-01 00:00:00	\N	0	1	\N
9	3bc319753945b6dba3e1928abed49e35	placeholder	4	0	true	\N	1970-01-01 00:00:00	\N	0	1	\N
10	01f6b1417935528fdc97ac2e130a150c	placeholder	4	0	true	\N	1970-01-01 00:00:00	\N	0	1	\N
11	bb43218d7d95b1fe062a731d1aa7e9e2	placeholder	4	0	true	\N	1970-01-01 00:00:00	\N	0	1	\N
12	f2b78b76f6093ba04204d972a7ce42d6	placeholder	4	0	true	\N	1970-01-01 00:00:00	\N	0	1	\N
13	e97809296efb670cff4f92138bd69ec8	placeholder	4	0	true	\N	1970-01-01 00:00:00	\N	0	1	\N
14	0fac88d8412f46d5e16beb87aee0fd22	date-posted	10	0	\N	\N	1970-01-01 00:00:00	\N	0	1	2018-02-20
\.


--
-- Data for Name: splits; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.splits (guid, tx_guid, account_guid, memo, action, reconcile_state, reconcile_date, value_num, value_denom, quantity_num, quantity_denom, lot_guid) FROM stdin;
de832fe97e37811a7fff7e28b3a43425	6c8876003c4a6026e38e3afb67d6f2b1	93fc043c3062aaa1297b30e543d2cd0d			n	1970-01-01 00:00:00	15000	100	15000	100	\N
1e612f650eb598d9e803902b6aca73e3	6c8876003c4a6026e38e3afb67d6f2b1	6bbc8f20544452cac1637fb9a9b851bb			n	1970-01-01 00:00:00	-15000	100	-15000	100	\N
42336a5732d54b22fa080be3b50ff5b7	0fac88d8412f46d5e16beb87aee0fd22	93fc043c3062aaa1297b30e543d2cd0d			n	1970-01-01 00:00:00	10000	100	10000	100	\N
486341b68457b037ecd75dcdcbf64bc5	0fac88d8412f46d5e16beb87aee0fd22	adc619f0ac7fa27d5768bfd73ecbc01e			n	1970-01-01 00:00:00	-10000	100	-10000	100	\N
eb832677cb2a013fcc4140316ccb3323	a5b50b770ee5e01361c49300aedb5e0f	af88d386d44b14acf244362b85ccaf4c			n	1970-01-01 00:00:00	3000	100	3000	100	\N
f32095185808aff854fc020ee2eedb5d	a5b50b770ee5e01361c49300aedb5e0f	93fc043c3062aaa1297b30e543d2cd0d			n	1970-01-01 00:00:00	-3000	100	-3000	100	\N
ea72c01205e1f619aa278930648b060e	e774d4b1fbd3c8c853b2c45a1d74f5f9	adc619f0ac7fa27d5768bfd73ecbc01e			n	1970-01-01 00:00:00	25000	100	25000	100	\N
bae8bc3a2b29812691016e61c425fda5	e774d4b1fbd3c8c853b2c45a1d74f5f9	a1dd9e8118bff87db289482bceebfea9			n	1970-01-01 00:00:00	-25000	100	-25000	100	\N
4f78ef3650c38115abf60365f9484b45	7b3a2df3645b3e490121814c8ad7146d	a1dd9e8118bff87db289482bceebfea9			n	1970-01-01 00:00:00	250000	100	250000	100	\N
afcba9bb98422e5ca64931a5efd86025	7b3a2df3645b3e490121814c8ad7146d	1305127e63737f8c39afb49b5bbeca7a			n	1970-01-01 00:00:00	-250000	100	-250000	100	\N
e07539176a70222b16369bd246b174b9	a5924cd14525c307cc5862c97361b031	1c089803052e85f5c6d8e786057dbaee			n	1970-01-01 00:00:00	120000	100	1300000	10000	\N
e055799a549308621a16c715c870e29a	a5924cd14525c307cc5862c97361b031	7894e9c3e955f5eaa9689d16ed775660			n	1970-01-01 00:00:00	120000	100	120000	100	\N
f5f1b767b38439eff788bed8fca8d1c2	a5924cd14525c307cc5862c97361b031	a1dd9e8118bff87db289482bceebfea9			n	1970-01-01 00:00:00	-120000	100	-120000	100	\N
85d451d09db85fb97cea6fe2e876d566	a5924cd14525c307cc5862c97361b031	0ccab772d0d16a3e1eaf42cd53f891e5			n	1970-01-01 00:00:00	-120000	100	-1300000	10000	\N
b3c4df045f2a19332e143cd83a7e013f	2d249b7d3fc8efe17865873b97748f50	a1dd9e8118bff87db289482bceebfea9			n	1970-01-01 00:00:00	250000	100	250000	100	\N
d240433d6812fc634c21200fb559d07f	2d249b7d3fc8efe17865873b97748f50	1305127e63737f8c39afb49b5bbeca7a			n	1970-01-01 00:00:00	-250000	100	-250000	100	\N
714c69b235d450f11e4add7077b4cde7	5be11c009a88b4198aa65ee878ddf99d	adc619f0ac7fa27d5768bfd73ecbc01e			n	1970-01-01 00:00:00	100000	100	100000	100	\N
3a53ab1f73cb74d9882dd14b7d6a4a78	5be11c009a88b4198aa65ee878ddf99d	96ed7a45459fb5fe570e48fcd46f05d0			n	1970-01-01 00:00:00	-100000	100	-100000	100	\N
b8387081c3d60310465a5d316fb6745a	6294086c678a584e7ce184b523699f6c	af88d386d44b14acf244362b85ccaf4c			n	1970-01-01 00:00:00	20000	100	20000	100	\N
d96c36fed7324332bfeff80e52cd9892	6294086c678a584e7ce184b523699f6c	adc619f0ac7fa27d5768bfd73ecbc01e			n	1970-01-01 00:00:00	-20000	100	-20000	100	\N
6ca852f33ee5415d22dfc132f5c5ec09	325537f4f0fadfd9ffb6aad3cd18e360	96ed7a45459fb5fe570e48fcd46f05d0	capital		n	1970-01-01 00:00:00	10000	100	10000	100	\N
3142f6ebed4469f9a692dfa69a0080f5	325537f4f0fadfd9ffb6aad3cd18e360	af88d386d44b14acf244362b85ccaf4c	interest		n	1970-01-01 00:00:00	3000	100	3000	100	\N
47c7fdca3bc04d6ad734eefc2794ddfe	325537f4f0fadfd9ffb6aad3cd18e360	adc619f0ac7fa27d5768bfd73ecbc01e	monthly payment		n	1970-01-01 00:00:00	-13000	100	-13000	100	\N
76cd1180d563db608495c710d2774c9a	59d9b60221a4615e74f8e489dceff58b	a3dc764fea53b709af7fcead6470da43			n	1970-01-01 00:00:00	2000000	100	2000000	100	\N
c8eb859a307babbb2fdf66871cf4c0a8	59d9b60221a4615e74f8e489dceff58b	96ed7a45459fb5fe570e48fcd46f05d0			n	1970-01-01 00:00:00	-2000000	100	-2000000	100	\N
\.


--
-- Data for Name: taxtable_entries; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.taxtable_entries (id, taxtable, account, amount_num, amount_denom, type) FROM stdin;
\.


--
-- Data for Name: taxtables; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.taxtables (guid, name, refcount, invisible, parent) FROM stdin;
\.


--
-- Data for Name: transactions; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.transactions (guid, currency_guid, num, post_date, enter_date, description) FROM stdin;
6c8876003c4a6026e38e3afb67d6f2b1	346629655191dcf59a7e2c2a85b70f69		2014-12-24 10:59:00	2014-12-25 10:08:15	income 1
0fac88d8412f46d5e16beb87aee0fd22	346629655191dcf59a7e2c2a85b70f69		2018-02-20 10:59:00	2018-02-20 21:12:49	Transfer current
a5b50b770ee5e01361c49300aedb5e0f	346629655191dcf59a7e2c2a85b70f69		2018-02-20 10:59:00	2018-02-20 21:13:01	Purchase
e774d4b1fbd3c8c853b2c45a1d74f5f9	346629655191dcf59a7e2c2a85b70f69		2018-02-20 10:59:00	2018-02-20 21:13:30	transfer intra
7b3a2df3645b3e490121814c8ad7146d	346629655191dcf59a7e2c2a85b70f69		2018-02-20 10:59:00	2018-02-20 21:13:50	Opening Balance
a5924cd14525c307cc5862c97361b031	346629655191dcf59a7e2c2a85b70f69		2018-02-21 10:59:00	2018-02-21 06:58:44	buy foo
2d249b7d3fc8efe17865873b97748f50	346629655191dcf59a7e2c2a85b70f69		2018-02-21 10:59:00	2018-02-21 07:25:53	Opening Balance
5be11c009a88b4198aa65ee878ddf99d	346629655191dcf59a7e2c2a85b70f69		2014-12-24 10:59:00	2014-12-25 10:07:30	initial load
6294086c678a584e7ce184b523699f6c	346629655191dcf59a7e2c2a85b70f69		2014-12-24 10:59:00	2014-12-25 10:08:08	expense 1
325537f4f0fadfd9ffb6aad3cd18e360	346629655191dcf59a7e2c2a85b70f69		2014-12-24 10:59:00	2014-12-25 10:11:26	loan payment
59d9b60221a4615e74f8e489dceff58b	346629655191dcf59a7e2c2a85b70f69		2018-02-20 10:59:00	2018-02-20 21:14:36	house load
\.


--
-- Data for Name: vendors; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.vendors (guid, name, id, notes, currency, active, tax_override, addr_name, addr_addr1, addr_addr2, addr_addr3, addr_addr4, addr_phone, addr_fax, addr_email, terms, tax_inc, tax_table) FROM stdin;
\.


--
-- Data for Name: versions; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.versions (table_name, table_version) FROM stdin;
Gnucash	4000005
Gnucash-Resave	19920
books	1
commodities	1
accounts	1
budgets	1
budget_amounts	1
prices	3
transactions	4
splits	5
slots	4
recurrences	2
schedxactions	1
lots	2
billterms	2
customers	2
employees	2
entries	4
invoices	4
jobs	1
orders	1
taxtables	2
taxtable_entries	3
vendors	1
\.


--
-- Name: budget_amounts_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.budget_amounts_id_seq', 1, false);


--
-- Name: recurrences_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.recurrences_id_seq', 1, false);


--
-- Name: slots_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.slots_id_seq', 14, true);


--
-- Name: taxtable_entries_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.taxtable_entries_id_seq', 1, false);


--
-- Name: accounts accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT accounts_pkey PRIMARY KEY (guid);


--
-- Name: billterms billterms_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.billterms
    ADD CONSTRAINT billterms_pkey PRIMARY KEY (guid);


--
-- Name: books books_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.books
    ADD CONSTRAINT books_pkey PRIMARY KEY (guid);


--
-- Name: budget_amounts budget_amounts_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.budget_amounts
    ADD CONSTRAINT budget_amounts_pkey PRIMARY KEY (id);


--
-- Name: budgets budgets_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.budgets
    ADD CONSTRAINT budgets_pkey PRIMARY KEY (guid);


--
-- Name: commodities commodities_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.commodities
    ADD CONSTRAINT commodities_pkey PRIMARY KEY (guid);


--
-- Name: customers customers_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.customers
    ADD CONSTRAINT customers_pkey PRIMARY KEY (guid);


--
-- Name: employees employees_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.employees
    ADD CONSTRAINT employees_pkey PRIMARY KEY (guid);


--
-- Name: entries entries_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.entries
    ADD CONSTRAINT entries_pkey PRIMARY KEY (guid);


--
-- Name: invoices invoices_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.invoices
    ADD CONSTRAINT invoices_pkey PRIMARY KEY (guid);


--
-- Name: jobs jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.jobs
    ADD CONSTRAINT jobs_pkey PRIMARY KEY (guid);


--
-- Name: lots lots_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.lots
    ADD CONSTRAINT lots_pkey PRIMARY KEY (guid);


--
-- Name: orders orders_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (guid);


--
-- Name: prices prices_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.prices
    ADD CONSTRAINT prices_pkey PRIMARY KEY (guid);


--
-- Name: recurrences recurrences_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.recurrences
    ADD CONSTRAINT recurrences_pkey PRIMARY KEY (id);


--
-- Name: schedxactions schedxactions_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.schedxactions
    ADD CONSTRAINT schedxactions_pkey PRIMARY KEY (guid);


--
-- Name: slots slots_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.slots
    ADD CONSTRAINT slots_pkey PRIMARY KEY (id);


--
-- Name: splits splits_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.splits
    ADD CONSTRAINT splits_pkey PRIMARY KEY (guid);


--
-- Name: taxtable_entries taxtable_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.taxtable_entries
    ADD CONSTRAINT taxtable_entries_pkey PRIMARY KEY (id);


--
-- Name: taxtables taxtables_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.taxtables
    ADD CONSTRAINT taxtables_pkey PRIMARY KEY (guid);


--
-- Name: transactions transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_pkey PRIMARY KEY (guid);


--
-- Name: vendors vendors_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.vendors
    ADD CONSTRAINT vendors_pkey PRIMARY KEY (guid);


--
-- Name: versions versions_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.versions
    ADD CONSTRAINT versions_pkey PRIMARY KEY (table_name);


--
-- Name: slots_guid_index; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX slots_guid_index ON public.slots USING btree (obj_guid);


--
-- Name: splits_account_guid_index; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX splits_account_guid_index ON public.splits USING btree (account_guid);


--
-- Name: splits_tx_guid_index; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX splits_tx_guid_index ON public.splits USING btree (tx_guid);


--
-- Name: tx_post_date_index; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX tx_post_date_index ON public.transactions USING btree (post_date);


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: postgres
--

GRANT USAGE ON SCHEMA public TO "user";


--
-- Name: TABLE accounts; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.accounts TO "user";


--
-- Name: TABLE billterms; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.billterms TO "user";


--
-- Name: TABLE books; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.books TO "user";


--
-- Name: TABLE budget_amounts; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.budget_amounts TO "user";


--
-- Name: TABLE budgets; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.budgets TO "user";


--
-- Name: TABLE commodities; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.commodities TO "user";


--
-- Name: TABLE customers; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.customers TO "user";


--
-- Name: TABLE employees; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.employees TO "user";


--
-- Name: TABLE entries; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.entries TO "user";


--
-- Name: TABLE gnclock; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.gnclock TO "user";


--
-- Name: TABLE invoices; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.invoices TO "user";


--
-- Name: TABLE jobs; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.jobs TO "user";


--
-- Name: TABLE lots; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.lots TO "user";


--
-- Name: TABLE orders; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.orders TO "user";


--
-- Name: TABLE prices; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.prices TO "user";


--
-- Name: TABLE recurrences; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.recurrences TO "user";


--
-- Name: TABLE schedxactions; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.schedxactions TO "user";


--
-- Name: TABLE slots; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.slots TO "user";


--
-- Name: TABLE splits; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.splits TO "user";


--
-- Name: TABLE taxtable_entries; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.taxtable_entries TO "user";


--
-- Name: TABLE taxtables; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.taxtables TO "user";


--
-- Name: TABLE transactions; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.transactions TO "user";


--
-- Name: TABLE vendors; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.vendors TO "user";


--
-- Name: TABLE versions; Type: ACL; Schema: public; Owner: postgres
--

GRANT SELECT ON TABLE public.versions TO "user";


--
-- PostgreSQL database dump complete
--

