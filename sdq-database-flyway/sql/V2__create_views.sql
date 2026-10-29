CREATE OR REPLACE VIEW client_full AS
WITH client AS (
    SELECT * FROM client
),
intervention_types AS (
    SELECT
        it.client_id,
        jsonb_agg(
            jsonb_build_object(
                'type', it.intervention_type,
                'sessions', it.sessions
            )
        ) AS interventions
    FROM intervention_type it
    GROUP BY it.client_id
),
disability_types AS (
    SELECT
        dt.client_id,
        jsonb_agg(dt.disability_type) AS disability_types
    FROM disability_type dt
    GROUP BY dt.client_id
),
aces AS (
    SELECT
        a.client_id,
        jsonb_object_agg(a.ace_type, a.score) AS aces
    FROM aces a
    GROUP BY a.client_id
)
SELECT
    c.*,
    it.interventions,
    dt.disability_types,
    a.aces
FROM client c
LEFT JOIN intervention_types it ON it.client_id = c.client_id
LEFT JOIN disability_types dt ON dt.client_id = c.client_id
LEFT JOIN aces a ON a.client_id = c.client_id;

CREATE OR REPLACE VIEW sdq_summary_view AS
WITH base AS (
    SELECT
        cl.client_id,
        cl.date_of_birth,
        cl.gender,
        cl.council,
        cl.ethnicity,
        cl.eal,
        cl.disability_status,
        cl.care_experience,
        cl.funding_source,
        s.assessor,
        p.period_id,
        p.period_date,
        s.statement AS statement_key,
        st.category,
        c.posture,
        s.score
    FROM sdq s
    INNER JOIN reporting_period p ON p.period_id = s.period_id
    INNER JOIN client cl ON cl.client_id = p.client_id
    INNER JOIN sdq_statement st ON st.statement_key = s.statement
    INNER JOIN sdq_category c ON c.category = st.category
),
category_totals AS (
    SELECT client_id, period_id, category, SUM(score) AS total
    FROM base
    GROUP BY client_id, period_id, category
),
posture_totals AS (
    SELECT client_id, period_id, posture, SUM(score) AS total
    FROM base
    GROUP BY client_id, period_id, posture
),
total_difficulties AS (
    SELECT client_id, period_id, SUM(score) AS total
    FROM base
    WHERE posture <> 'ProSocial'
    GROUP BY client_id, period_id
)
SELECT
    b.client_id,
    b.period_id,
    b.period_date,
    ANY_VALUE(b.date_of_birth) AS date_of_birth,
    ANY_VALUE(b.gender) AS gender,
    ANY_VALUE(b.council) AS council,
    ANY_VALUE(b.ethnicity) AS ethnicity,
    ANY_VALUE(b.eal) AS eal,
    ANY_VALUE(b.disability_status) AS disability_status,
    ANY_VALUE(b.care_experience) AS care_experience,
    ANY_VALUE(b.funding_source) AS funding_source,
    b.assessor,
    (
        SELECT JSONB_OBJECT_AGG(category, total)
        FROM category_totals ct
        WHERE ct.client_id = b.client_id
          AND ct.period_id = b.period_id
    ) AS category_subtotals,
    (
        SELECT JSONB_OBJECT_AGG(posture, total)
        FROM posture_totals pt
        WHERE pt.client_id = b.client_id
          AND pt.period_id = b.period_id
    ) AS posture_subtotals,
    (
        SELECT total
        FROM total_difficulties td
        WHERE td.client_id = b.client_id
          AND td.period_id = b.period_id
    ) AS total_difficulties
FROM base b
GROUP BY b.client_id, b.period_id, b.period_date, b.assessor
ORDER BY b.client_id, b.period_date, b.assessor;

CREATE OR REPLACE VIEW sdq_single_summary_view AS
WITH base AS (
    SELECT
        s.period_id,
        p.period_date,
        s.assessor,
        s.statement AS statement_key,
        st.category,
        s.score,
        c.posture
    FROM sdq s
    INNER JOIN sdq_statement st ON st.statement_key = s.statement
    INNER JOIN sdq_category c ON c.category = st.category
    INNER JOIN reporting_period p ON p.period_id = s.period_id
),
period_info AS (
    SELECT DISTINCT period_id, period_date, assessor
    FROM base
),
category_totals AS (
    SELECT period_id, assessor, category, SUM(score) AS total
    FROM base
    GROUP BY period_id, assessor, category
),
posture_totals AS (
    SELECT period_id, assessor, posture, SUM(score) AS total
    FROM base
    GROUP BY period_id, assessor, posture
),
total_difficulties AS (
    SELECT period_id, assessor, SUM(score) AS total
    FROM base
    WHERE posture <> 'ProSocial'
    GROUP BY period_id, assessor
),
period_summary AS (
    SELECT
        pi.period_id,
        pi.period_date,
        pi.assessor,
        (
            SELECT JSONB_OBJECT_AGG(category, total)
            FROM category_totals
            WHERE period_id = pi.period_id
              AND assessor = pi.assessor
        ) AS category_subtotals,
        (
            SELECT JSONB_OBJECT_AGG(posture, total)
            FROM posture_totals
            WHERE period_id = pi.period_id
              AND assessor = pi.assessor
        ) AS posture_subtotals,
        (
            SELECT total
            FROM total_difficulties
            WHERE period_id = pi.period_id
              AND assessor = pi.assessor
        ) AS total_difficulties
    FROM period_info pi
    WHERE period_id = pi.period_id
      AND assessor = pi.assessor
)
SELECT * FROM period_summary;

CREATE OR REPLACE VIEW goal_progress_view AS
SELECT DISTINCT
    g.client_id,
    o.goal_id,
    g.type AS goal_type,
    g.description AS goal_description,
    o.assessor,
    FIRST_VALUE(o.period_date) OVER (
        PARTITION BY g.client_id, o.goal_id, o.assessor
        ORDER BY o.period_date
    ) AS first_period_date,
    FIRST_VALUE(o.score) OVER (
        PARTITION BY g.client_id, o.goal_id, o.assessor
        ORDER BY o.period_date
    ) AS first_score,
    LAST_VALUE(o.period_date) OVER (
        PARTITION BY g.client_id, o.goal_id, o.assessor
        ORDER BY o.period_date
        RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
    ) AS last_period_date,
    LAST_VALUE(o.score) OVER (
        PARTITION BY g.client_id, o.goal_id, o.assessor
        ORDER BY o.period_date
        RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
    ) AS last_score
FROM gbo_score o
JOIN goal g ON g.goal_id = o.goal_id;

CREATE OR REPLACE VIEW sdq_progress_view AS
WITH base AS (
    SELECT
        cl.client_id,
        cl.date_of_birth,
        cl.gender,
        cl.council,
        cl.ethnicity,
        cl.eal,
        cl.disability_status,
        cl.care_experience,
        cl.funding_source,
        p.period_date,
        s.statement AS statement_key,
        st.category,
        c.posture,
        s.score,
        s.assessor
    FROM sdq s
    INNER JOIN reporting_period p ON p.period_id = s.period_id
    INNER JOIN client cl ON cl.client_id = p.client_id
    INNER JOIN sdq_statement st ON st.statement_key = s.statement
    INNER JOIN sdq_category c ON c.category = st.category
),
posture_totals AS (
    SELECT client_id, assessor, posture, period_date, SUM(score) AS total
    FROM base
    GROUP BY client_id, assessor, posture, period_date
),
category_totals AS (
    SELECT client_id, assessor, category, period_date, SUM(score) AS total
    FROM base
    GROUP BY client_id, assessor, category, period_date
),
total_difficulties AS (
    SELECT client_id, assessor, period_date, SUM(score) AS total
    FROM base
    WHERE posture <> 'ProSocial'
    GROUP BY client_id, assessor, period_date
),
category_progress AS (
    SELECT
        client_id,
        assessor,
        category,
        total,
        FIRST_VALUE(total) OVER (
            PARTITION BY client_id, assessor, category
            ORDER BY period_date
        ) AS first_total,
        LAST_VALUE(total) OVER (
            PARTITION BY client_id, assessor, category
            ORDER BY period_date
            RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
        ) AS last_total
    FROM category_totals
),
posture_progress AS (
    SELECT
        client_id,
        assessor,
        posture,
        total,
        FIRST_VALUE(total) OVER (
            PARTITION BY client_id, assessor, posture
            ORDER BY period_date
        ) AS first_total,
        LAST_VALUE(total) OVER (
            PARTITION BY client_id, assessor, posture
            ORDER BY period_date
            RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
        ) AS last_total
    FROM posture_totals
),
total_progress AS (
    SELECT
        client_id,
        assessor,
        total,
        FIRST_VALUE(total) OVER (
            PARTITION BY client_id, assessor
            ORDER BY period_date
        ) AS first_total,
        LAST_VALUE(total) OVER (
            PARTITION BY client_id, assessor
            ORDER BY period_date
            RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
        ) AS last_total
    FROM total_difficulties
),
category_final AS (
    SELECT
        client_id,
        assessor,
        category,
        MIN(first_total) AS first_total,
        MAX(last_total) AS last_total,
        MAX(last_total) - MIN(first_total) AS delta
    FROM category_progress
    GROUP BY client_id, assessor, category
),
posture_final AS (
    SELECT
        client_id,
        assessor,
        posture,
        MIN(first_total) AS first_total,
        MAX(last_total) AS last_total,
        MAX(last_total) - MIN(first_total) AS delta
    FROM posture_progress
    GROUP BY client_id, assessor, posture
),
total_final AS (
    SELECT
        client_id,
        assessor,
        MIN(first_total) AS first_total,
        MAX(last_total) AS last_total,
        MAX(last_total) - MIN(first_total) AS delta
    FROM total_progress
    GROUP BY client_id, assessor
),
progress_summary AS (
    SELECT
        client_id,
        cf.assessor AS assessor,
        JSONB_OBJECT_AGG(
            pf.posture,
            JSONB_BUILD_OBJECT(
                'first', pf.first_total,
                'last', pf.last_total,
                'delta', pf.delta
            )
        ) AS posture_progress,
        JSONB_OBJECT_AGG(
            category,
            JSONB_BUILD_OBJECT(
                'first', cf.first_total,
                'last', cf.last_total,
                'delta', cf.delta
            )
        ) AS category_progress,
        (
            SELECT JSONB_BUILD_OBJECT(
                'first', tf.first_total,
                'last', tf.last_total,
                'delta', tf.delta
            )
            FROM total_final tf
            WHERE tf.client_id = cf.client_id
              AND tf.assessor = cf.assessor
        ) AS total_progress
    FROM category_final cf
    JOIN posture_final pf USING (client_id)
    JOIN total_final tf USING (client_id)
    GROUP BY client_id, cf.assessor
)
SELECT
    p.client_id,
    p.assessor,
    b.date_of_birth,
    b.gender,
    b.council,
    b.ethnicity,
    b.eal,
    b.disability_status,
    b.care_experience,
    b.funding_source,
    b.first_period_date,
    b.last_period_date,
    p.posture_progress,
    p.category_progress,
    p.total_progress
FROM progress_summary p
JOIN (
    SELECT
        client_id,
        ANY_VALUE(date_of_birth) AS date_of_birth,
        ANY_VALUE(gender) AS gender,
        ANY_VALUE(council) AS council,
        ANY_VALUE(ethnicity) AS ethnicity,
        ANY_VALUE(eal) AS eal,
        ANY_VALUE(disability_status) AS disability_status,
        ANY_VALUE(care_experience) AS care_experience,
        ANY_VALUE(funding_source) AS funding_source,
        MIN(period_date) AS first_period_date,
        MAX(period_date) AS last_period_date
    FROM base
    GROUP BY client_id
) b ON p.client_id = b.client_id;
