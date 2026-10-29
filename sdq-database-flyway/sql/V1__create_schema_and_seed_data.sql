CREATE TABLE client (
    client_id UUID PRIMARY KEY,
    code_name TEXT NOT NULL,
    date_of_birth DATE,
    gender TEXT,
    council TEXT,
    ethnicity TEXT,
    eal TEXT,
    disability_status TEXT,
    care_experience TEXT,
    funding_source TEXT
);

CREATE TABLE goal (
    goal_id UUID PRIMARY KEY,
    type TEXT,
    client_id UUID NOT NULL,
    description TEXT,
    CONSTRAINT fk_client_goal
        FOREIGN KEY (client_id) REFERENCES client (client_id) ON DELETE CASCADE
);

CREATE TABLE gbo_score (
    goal_id UUID NOT NULL,
    assessor TEXT NOT NULL,
    period_date DATE NOT NULL,
    score INT,
    CONSTRAINT pk_gbo_score PRIMARY KEY (goal_id, assessor, period_date),
    CONSTRAINT fk_goal_score
        FOREIGN KEY (goal_id) REFERENCES goal (goal_id) ON DELETE CASCADE
);

CREATE TABLE intervention_type (
    client_id UUID NOT NULL,
    intervention_type TEXT NOT NULL,
    sessions INT,
    CONSTRAINT fk_client_intervention_type
        FOREIGN KEY (client_id) REFERENCES client (client_id) ON DELETE CASCADE
);
CREATE INDEX idx_intervention_type_client_id ON intervention_type (client_id);

CREATE TABLE disability_type (
    client_id UUID NOT NULL,
    disability_type TEXT NOT NULL,
    CONSTRAINT fk_client_disability_type
        FOREIGN KEY (client_id) REFERENCES client (client_id) ON DELETE CASCADE
);
CREATE INDEX idx_disability_type_client_id ON disability_type (client_id);

CREATE TABLE aces (
    client_id UUID NOT NULL,
    ace_type TEXT NOT NULL,
    score INT,
    CONSTRAINT fk_client_aces
        FOREIGN KEY (client_id) REFERENCES client (client_id) ON DELETE CASCADE
);
CREATE INDEX idx_aces_client_id ON aces (client_id);

CREATE TABLE reporting_period (
    period_id UUID PRIMARY KEY,
    client_id UUID NOT NULL,
    period_date DATE,
    CONSTRAINT fk_client_reporting_period
        FOREIGN KEY (client_id) REFERENCES client (client_id) ON DELETE CASCADE
);

CREATE TABLE sdq_category (
    category VARCHAR(50) PRIMARY KEY,
    posture VARCHAR(50) NOT NULL
);

INSERT INTO sdq_category (category, posture) VALUES
    ('Emotional', 'Internalising'),
    ('Peer', 'Internalising'),
    ('Conduct', 'Externalising'),
    ('HyperActivity', 'Externalising'),
    ('ProSocial', 'ProSocial');

CREATE TABLE sdq_statement (
    "order" INT NOT NULL,
    statement_key VARCHAR(100) PRIMARY KEY,
    category VARCHAR(50) NOT NULL,
    description VARCHAR(255),
    is_true_positive BOOLEAN NOT NULL,
    CONSTRAINT fk_statement_category
        FOREIGN KEY (category) REFERENCES sdq_category (category)
);

INSERT INTO sdq_statement ("order", statement_key, category, is_true_positive, description) VALUES
    (1, 'CONSIDERATE', 'ProSocial', FALSE, 'Considerate of other people’s feelings'),
    (2, 'RESTLESS', 'HyperActivity', FALSE, 'Restless, overactive, cannot stay still for long'),
    (3, 'COMPLAINS_ACHES', 'Emotional', FALSE, 'Often complains of headaches, stomach-aches or sickness'),
    (4, 'SHARES_READILY', 'ProSocial', FALSE, 'Shares readily with other children (treats, toys, pencils etc)'),
    (5, 'TEMPER', 'Conduct', FALSE, 'Often has temper tantrums or hot tempers'),
    (6, 'SOLITARY', 'Peer', FALSE, 'Rather solitary, tends to play alone'),
    (7, 'OBEDIENT', 'Conduct', TRUE, 'Generally obedient, usually does what adults request'),
    (8, 'WORRIES', 'Emotional', FALSE, 'Many worries, often seems worried'),
    (9, 'HELPFUL', 'ProSocial', FALSE, 'Helpful if someone is hurt, upset or feeling ill'),
    (10, 'FIDGETING', 'HyperActivity', FALSE, 'Constantly fidgeting or squirming'),
    (11, 'ONE_GOOD_FRIEND', 'Peer', TRUE, 'Has at least one good friend'),
    (12, 'FIGHTS', 'Conduct', FALSE, 'Often fights with other children or bullies them'),
    (13, 'UNHAPPY', 'Emotional', FALSE, 'Often unhappy, down-hearted or tearful'),
    (14, 'LIKED_BY_OTHERS', 'Peer', TRUE, 'Generally liked by other children'),
    (15, 'DISTRACTED', 'HyperActivity', FALSE, 'Easily distracted, concentration wanders'),
    (16, 'NERVOUS', 'Emotional', FALSE, 'Nervous or clingy in new situations, easily loses confidence'),
    (17, 'KIND_TO_YOUNGER', 'ProSocial', FALSE, 'Kind to younger children'),
    (18, 'LIES', 'Conduct', FALSE, 'Often lies or cheats'),
    (19, 'PICKED_ON', 'Peer', FALSE, 'Picked on or bullied by other children'),
    (20, 'VOLUNTEERS', 'ProSocial', FALSE, 'Often volunteers to help others (parents, teachers, other children)'),
    (21, 'THINKS_THROUGH', 'HyperActivity', TRUE, 'Thinks things out before acting'),
    (22, 'STEALS', 'Conduct', FALSE, 'Steals from home, school or elsewhere'),
    (23, 'GETS_ON_ADULTS_BETTER', 'Peer', FALSE, 'Gets on better with adults than with other children'),
    (24, 'FEARS', 'Emotional', FALSE, 'Many fears, easily scared'),
    (25, 'ATTENTION', 'HyperActivity', TRUE, 'Sees tasks through to the end, good attention span');

CREATE TABLE sdq (
    period_id UUID NOT NULL,
    assessor TEXT NOT NULL,
    statement TEXT NOT NULL,
    score INT,
    CONSTRAINT pk_sdq PRIMARY KEY (period_id, assessor, statement),
    CONSTRAINT fk_sdq_statement
        FOREIGN KEY (statement) REFERENCES sdq_statement (statement_key),
    CONSTRAINT fk_sdq_reporting_period
        FOREIGN KEY (period_id) REFERENCES reporting_period (period_id) ON DELETE CASCADE
);
