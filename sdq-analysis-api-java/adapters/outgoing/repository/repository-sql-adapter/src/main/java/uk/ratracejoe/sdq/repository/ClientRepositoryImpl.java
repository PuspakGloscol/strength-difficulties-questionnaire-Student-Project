package uk.ratracejoe.sdq.repository;

import static uk.ratracejoe.sdq.repository.RepositoryJsonUtils.parseJson;
import static uk.ratracejoe.sdq.repository.RepositoryJsonUtils.whereClause;

import com.fasterxml.jackson.core.type.TypeReference;
import java.sql.Date;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.*;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import org.springframework.jdbc.core.simple.JdbcClient;
import uk.ratracejoe.sdq.exception.SdqException;
import uk.ratracejoe.sdq.model.SdqClient;
import uk.ratracejoe.sdq.model.demographics.*;

public class ClientRepositoryImpl implements ClientRepository {
  private final JdbcClient jdbcClient;
  private final DisabilityTypeRepositoryImpl disabilityTypeRepository;
  private final InterventionRepositoryImpl interventionRepository;
  private final AcesRepositoryImpl acesRepository;

  public ClientRepositoryImpl(JdbcClient jdbcClient) {
    this.jdbcClient = jdbcClient;
    this.disabilityTypeRepository = new DisabilityTypeRepositoryImpl(jdbcClient);
    this.interventionRepository = new InterventionRepositoryImpl(jdbcClient);
    this.acesRepository = new AcesRepositoryImpl(jdbcClient);
  }

  public SdqClient createClient(SdqClient client) throws SdqException {
    UUID clientId = Optional.ofNullable(client.clientId()).orElseGet(UUID::randomUUID);

    List<String> fields = new ArrayList<>();
    Map<String, Object> params = new HashMap<>();
    Function<String, Consumer<Object>> addField =
        (fieldName) ->
            v -> {
              params.put(fieldName, v);
              fields.add(fieldName);
            };
    addField.apply("client_id").accept(clientId);
    addField.apply("code_name").accept(client.codeName());
    addField.apply("date_of_birth").accept(Date.valueOf(client.dateOfBirth()));

    Optional.ofNullable(client.gender()).map(Gender::name).ifPresent(addField.apply("gender"));
    Optional.ofNullable(client.council()).map(Council::name).ifPresent(addField.apply("council"));
    Optional.ofNullable(client.ethnicity())
        .map(Ethnicity::name)
        .ifPresent(addField.apply("ethnicity"));
    Optional.ofNullable(client.englishAdditionalLanguage())
        .map(EnglishAsAdditionalLanguage::name)
        .ifPresent(addField.apply("eal"));
    Optional.ofNullable(client.disabilityStatus())
        .map(DisabilityStatus::name)
        .ifPresent(addField.apply("disability_status"));
    Optional.ofNullable(client.careExperience())
        .map(CareExperience::name)
        .ifPresent(addField.apply("care_experience"));
    Optional.ofNullable(client.fundingSource())
        .map(FundingSource::name)
        .ifPresent(addField.apply("funding_source"));

    StringBuilder sql = new StringBuilder();
    sql.append("INSERT INTO client (");
    String fieldNames = String.join(",", fields);
    sql.append(fieldNames);
    sql.append(") VALUES (");
    String placeholders = fields.stream().map(f -> ":" + f).collect(Collectors.joining(", "));
    sql.append(placeholders);
    sql.append(")");
    jdbcClient.sql(sql.toString()).params(params).update();

    Optional.ofNullable(client.interventions())
        .ifPresent(its -> its.forEach(it -> interventionRepository.save(clientId, it)));
    Optional.ofNullable(client.disabilityTypes())
        .ifPresent(dts -> dts.forEach(dt -> disabilityTypeRepository.save(clientId, dt)));
    Optional.ofNullable(client.aces())
        .ifPresent(aces -> aces.forEach((key, value) -> acesRepository.save(clientId, key, value)));
    return get(clientId);
  }

  @Override
  public DemographicReport getDemographicReport(DemographicField demographic) {
    String sql = getDemographicReportSQL(demographic);

    List<DemographicCount> counts =
        jdbcClient
            .sql(sql)
            .query(
                (rs, rowNum) ->
                    new DemographicCount(rs.getString(1), rs.getInt(2), rs.getDouble(3)))
            .list();

    return new DemographicReport(counts);
  }

  @Override
  public SdqClient get(UUID clientId) {
    return jdbcClient
        .sql("SELECT * FROM client_full WHERE client_id=:clientId")
        .param("clientId", clientId) // positional parameter
        .query((rs, rowNum) -> getFromResultSet(rs))
        .single();
  }

  public List<SdqClient> getAll() throws SdqException {
    return jdbcClient
        .sql("SELECT * from client_full")
        .query((rs, rowNum) -> getFromResultSet(rs))
        .list();
  }

  @Override
  public List<SdqClient> getFiltered(String partialName, List<DemographicFilter> filters) {
    // Build SQL dynamically based on fields
    String sql = selectFilteredSql(partialName, filters.stream().toList());

    // Bind positional parameters
    Map<String, Object> params = new HashMap<>();
    addFilters(params, filters);
    if (partialName != null) {
      params.put("partial_name", partialName);
    }

    // Execute + map
    return jdbcClient.sql(sql).params(params).query((rs, rowNum) -> getFromResultSet(rs)).list();
  }

  private SdqClient getFromResultSet(ResultSet rs) throws SQLException {
    UUID clientId = rs.getObject("client_id", UUID.class);
    String codeName = rs.getString("code_name");
    Date dob = rs.getDate("date_of_birth");
    Gender gender =
        Optional.ofNullable(rs.getString("gender"))
            .map(Gender::valueOf)
            .orElseGet(Gender::defaultValue);
    Council council =
        Optional.ofNullable(rs.getString("council"))
            .map(Council::valueOf)
            .orElseGet(Council::defaultValue);
    Ethnicity ethnicity =
        Optional.ofNullable(rs.getString("ethnicity"))
            .map(Ethnicity::valueOf)
            .orElseGet(Ethnicity::defaultValue);
    EnglishAsAdditionalLanguage eal =
        Optional.ofNullable(rs.getString("eal"))
            .map(EnglishAsAdditionalLanguage::valueOf)
            .orElseGet(EnglishAsAdditionalLanguage::defaultValue);
    DisabilityStatus disabilityStatus =
        Optional.ofNullable(rs.getString("disability_status"))
            .map(DisabilityStatus::valueOf)
            .orElseGet(DisabilityStatus::defaultValue);
    CareExperience careExperience =
        Optional.ofNullable(rs.getString("care_experience"))
            .map(CareExperience::valueOf)
            .orElseGet(CareExperience::defaultValue);
    FundingSource fundingSource =
        Optional.ofNullable(rs.getString("funding_source"))
            .map(FundingSource::valueOf)
            .orElseGet(FundingSource::defaultValue);

    String interventionsJson = rs.getString("interventions");
    List<Intervention> interventions =
        parseJson(interventionsJson, new TypeReference<>() {}, Collections::emptyList);
    String disabilityTypesJson = rs.getString("disability_types");
    List<DisabilityType> disabilityTypes =
        parseJson(disabilityTypesJson, new TypeReference<>() {}, Collections::emptyList);

    String acesJson = rs.getString("aces");
    Map<AceType, Integer> aces =
        parseJson(acesJson, new TypeReference<>() {}, Collections::emptyMap);

    return new SdqClient(
        clientId,
        codeName,
        dob.toLocalDate(),
        gender,
        council,
        ethnicity,
        eal,
        disabilityStatus,
        disabilityTypes,
        careExperience,
        interventions,
        aces,
        fundingSource);
  }

  public int deleteAll() {
    return jdbcClient.sql("DELETE FROM client").update();
  }

  @Override
  public int update(SdqClient client) {
    SdqClient existing = get(client.clientId());

    interventionRepository.deleteForClient(client.clientId());
    disabilityTypeRepository.deleteForClient(client.clientId());
    acesRepository.deleteForClient(client.clientId());
    Optional.ofNullable(client.interventions())
        .ifPresent(its -> its.forEach(it -> interventionRepository.save(client.clientId(), it)));
    Optional.ofNullable(client.disabilityTypes())
        .ifPresent(dts -> dts.forEach(dt -> disabilityTypeRepository.save(client.clientId(), dt)));
    Optional.ofNullable(client.aces())
        .ifPresent(
            aces ->
                aces.forEach((key, value) -> acesRepository.save(client.clientId(), key, value)));

    return jdbcClient
        .sql(updateSQL())
        .param("code_name", Optional.ofNullable(client.codeName()).orElseGet(existing::codeName))
        .param(
            "date_of_birth",
            Optional.ofNullable(client.dateOfBirth()).orElseGet(existing::dateOfBirth))
        .param("gender", Optional.ofNullable(client.gender()).orElseGet(existing::gender).name())
        .param("council", Optional.ofNullable(client.council()).orElseGet(existing::council).name())
        .param(
            "ethnicity",
            Optional.ofNullable(client.ethnicity()).orElseGet(existing::ethnicity).name())
        .param(
            "eal",
            Optional.ofNullable(client.englishAdditionalLanguage())
                .orElseGet(existing::englishAdditionalLanguage)
                .name())
        .param(
            "disability_status",
            Optional.ofNullable(client.disabilityStatus())
                .orElseGet(existing::disabilityStatus)
                .name())
        .param(
            "care_experience",
            Optional.ofNullable(client.careExperience()).orElseGet(existing::careExperience).name())
        .param(
            "funding_source",
            Optional.ofNullable(client.fundingSource()).orElseGet(existing::fundingSource).name())
        .param("client_id", client.clientId())
        .update();
  }

  @Override
  public int deleteByClientId(UUID clientId) {
    return jdbcClient
        .sql("DELETE FROM client c WHERE c.client_id = :client_id")
        .param("client_id", clientId)
        .update();
  }

  private static final List<String> UPDATE_FIELDS =
      List.of(
          "code_name",
          "date_of_birth",
          "gender",
          "council",
          "ethnicity",
          "eal",
          "disability_status",
          "care_experience",
          "funding_source");

  static String updateSQL() {
    StringBuilder sb = new StringBuilder();
    sb.append("UPDATE client SET ");
    String fieldNames =
        String.join(",", UPDATE_FIELDS.stream().map(f -> String.format("%s = :%s", f, f)).toList());
    sb.append(fieldNames);
    sb.append(" WHERE ");
    sb.append("client_id");
    sb.append("  = :client_id");
    return sb.toString();
  }

  static String demographicColumn(DemographicField field) {
    return switch (field) {
      case Gender -> "gender";
      case Council -> "council";
      case Ethnicity -> "ethnicity";
      case EAL -> "eal";
      case DisabilityStatus -> "disability_status";
      case CareExperience -> "care_experience";
      case FundingSource -> "funding_source";
      case DisabilityType, InterventionType, ACES ->
          throw new IllegalArgumentException(
              "Demographic field is not currently supported in database queries: " + field);
    };
  }

  static String getDemographicReportSQL(DemographicField demographic) {
    String columnName = demographicColumn(demographic);
    return String.format(
        "select %s, count(*) AS count, round(100 * count(*) / (select count(*) from client), 2) as percentage FROM client GROUP BY %s;",
        columnName, columnName);
  }

  static String selectSQL(String clientSelectSql) {
    return String.format(
        """
        WITH client AS (
            %s
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
            JOIN client c ON c.client_id = it.client_id
            GROUP BY it.client_id
        ),
        disability_types AS (
            SELECT
                dt.client_id,
                jsonb_agg(dt.disability_type) AS disability_types
            FROM disability_type dt
            JOIN client c ON c.client_id = dt.client_id
            GROUP BY dt.client_id
        ),
        aces AS (
            SELECT
                a.client_id,
                jsonb_object_agg(a.ace_type, a.score) AS aces
            FROM aces a
            JOIN client c ON c.client_id = a.client_id
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
                """,
        clientSelectSql);
  }

  static String selectFilteredSql(String partialName, List<DemographicFilter> filters) {
    boolean filterOnName = partialName != null && !partialName.isEmpty();
    List<String> conditions = new ArrayList<>();
    if (!filters.isEmpty()) {
      filterSelectWhere("client", filters).forEach(conditions::add);
    }
    if (filterOnName) {
      conditions.add("code_name LIKE CONCAT('%', :partial_name, '%')");
    }
    return selectSQL(String.format("SELECT * FROM client %s", whereClause(conditions)));
  }

  public static void addFilters(Map<String, Object> params, List<DemographicFilter> filters) {
    for (DemographicFilter fv : filters) {
      if (isClientColumnField(fv.field())) {
        params.put(demographicColumn(fv.field()), fv.values());
      }
    }
  }

  public static boolean hasSupportedDemographicFilters(List<DemographicFilter> filters) {
    return filters.stream().anyMatch(filter -> isClientColumnField(filter.field()));
  }

  public static Stream<String> filterSelectWhere(
      String tableAlias, List<DemographicFilter> fields) {
    return fields.stream()
        .map(DemographicFilter::field)
        .filter(ClientRepositoryImpl::isClientColumnField)
        .map(ClientRepositoryImpl::demographicColumn)
        .map(column -> String.format("%s.%s IN (:%s)", tableAlias, column, column));
  }

  public static Stream<String> filterSelectWhere(List<DemographicFilter> fields) {
    return fields.stream()
        .map(DemographicFilter::field)
        .filter(ClientRepositoryImpl::isClientColumnField)
        .map(ClientRepositoryImpl::demographicColumn)
        .map(column -> String.format("%s IN (:%s)", column, column));
  }

  private static boolean isClientColumnField(DemographicField field) {
    return switch (field) {
      case Gender, Council, Ethnicity, EAL, DisabilityStatus, CareExperience, FundingSource -> true;
      case DisabilityType, InterventionType, ACES -> false;
    };
  }
}
