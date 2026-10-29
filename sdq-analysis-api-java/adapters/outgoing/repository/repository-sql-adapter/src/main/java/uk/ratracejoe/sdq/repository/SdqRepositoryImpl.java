package uk.ratracejoe.sdq.repository;

import static uk.ratracejoe.sdq.repository.RepositoryJsonUtils.parseJson;
import static uk.ratracejoe.sdq.repository.RepositoryJsonUtils.whereClause;

import com.fasterxml.jackson.core.type.TypeReference;
import java.time.LocalDate;
import java.util.*;
import lombok.RequiredArgsConstructor;
import org.springframework.jdbc.core.simple.JdbcClient;
import uk.ratracejoe.sdq.exception.SdqException;
import uk.ratracejoe.sdq.model.Assessor;
import uk.ratracejoe.sdq.model.demographics.DemographicFilter;
import uk.ratracejoe.sdq.model.sdq.*;

@RequiredArgsConstructor
public class SdqRepositoryImpl implements SdqRepository {
  private final JdbcClient jdbcClient;

  private final StatementRepository statementRepository;

  @Override
  public void save(SdqSubmission sdq) throws SdqException {
    sdq.scores()
        .forEach(
            score ->
                jdbcClient
                    .sql(
                        """
                INSERT INTO sdq
                  (period_id, assessor, statement, score)
                VALUES
                  (:periodId, :assessor, :statement, :score)""")
                    .param("periodId", sdq.periodId())
                    .param("assessor", sdq.assessor().name())
                    .param("statement", score.statement().key())
                    .param("score", score.score())
                    .update());
  }

  @Override
  public SdqSubmission get(UUID periodId, Assessor assessor) {
    return SdqSubmission.builder()
        .periodId(periodId)
        .assessor(assessor)
        .scores(getScores(periodId, assessor))
        .build();
  }

  public int deleteAll() throws SdqException {
    return jdbcClient.sql("DELETE FROM sdq").update();
  }

  @Override
  public SdqProgressSummary getProgressForClient(UUID clientId, Assessor assessor) {
    Map<String, Object> params = new HashMap<>();
    params.put("assessor", assessor.name());
    params.put("clientId", clientId);
    return jdbcClient
        .sql("SELECT * FROM sdq_progress_view WHERE client_id = :clientId AND assessor = :assessor")
        .params(params)
        .query(
            (rs, rowNum) -> {
              String categoryJson = rs.getString("category_progress");
              String postureJson = rs.getString("posture_progress");
              Map<String, Progress> byCategory =
                  parseJson(categoryJson, new TypeReference<>() {}, Collections::emptyMap);
              Map<Posture, Progress> byPosture =
                  convertPosture(
                      parseJson(postureJson, new TypeReference<>() {}, Collections::emptyMap));

              Progress totalDifficulties =
                  parseJson(rs.getString("total_progress"), Progress.class);

              return SdqProgressSummary.builder()
                  .clientId(clientId)
                  .assessor(assessor)
                  .categoryProgress(byCategory)
                  .postureProgress(byPosture)
                  .totalDifficulties(totalDifficulties)
                  .build();
            })
        .optional()
        .orElseThrow(() -> new SdqException("No progress found for client"));
  }

  @Override
  public List<SdqProgressSummary> getSdqProgress(
      List<Assessor> assessors, List<DemographicFilter> filters, LocalDate from, LocalDate to) {
    List<String> conditions = new ArrayList<>();
    conditions.add("assessor IN (:assessors)");
    if (ClientRepositoryImpl.hasSupportedDemographicFilters(filters)) {
      conditions.add("last_period_date >= :period_from");
      conditions.add("last_period_date < :period_to");
      ClientRepositoryImpl.filterSelectWhere(filters).forEach(conditions::add);
    }
    Map<String, Object> params = new HashMap<>();
    params.put("period_from", from);
    params.put("period_to", to);
    params.put("assessors", assessors.stream().map(Assessor::name).toList());
    ClientRepositoryImpl.addFilters(params, filters);
    return jdbcClient
        .sql(String.format("SELECT * FROM sdq_progress_view %s", whereClause(conditions)))
        .params(params)
        .query(
            (rs, rowNum) -> {
              UUID clientId = rs.getObject("client_id", UUID.class);
              String categoryJson = rs.getString("category_progress");
              String postureJson = rs.getString("posture_progress");
              Map<String, Progress> byCategory =
                  parseJson(categoryJson, new TypeReference<>() {}, Collections::emptyMap);
              Map<Posture, Progress> byPosture =
                  convertPosture(
                      parseJson(postureJson, new TypeReference<>() {}, Collections::emptyMap));

              Progress totalDifficulties =
                  parseJson(rs.getString("total_progress"), Progress.class);

              return SdqProgressSummary.builder()
                  .clientId(clientId)
                  .assessor(
                      Optional.ofNullable(rs.getString("assessor"))
                          .map(Assessor::valueOf)
                          .orElse(Assessor.Unknown))
                  .categoryProgress(byCategory)
                  .postureProgress(byPosture)
                  .totalDifficulties(totalDifficulties)
                  .build();
            })
        .list();
  }

  @Override
  public List<SdqSubmissionSummary> getFiltered(
      List<Assessor> assessors, List<DemographicFilter> filters, LocalDate from, LocalDate to) {
    List<String> conditions = new ArrayList<>();
    conditions.add("assessor IN (:assessors)");
    conditions.add("period_date >= :period_from");
    conditions.add("period_date < :period_to");
    if (!filters.isEmpty()) {
      ClientRepositoryImpl.filterSelectWhere(filters).forEach(conditions::add);
    }
    Map<String, Object> params = new HashMap<>();
    params.put("period_from", from);
    params.put("period_to", to);
    params.put("assessors", assessors.stream().map(Assessor::name).toList());
    ClientRepositoryImpl.addFilters(params, filters);
    return jdbcClient
        .sql(String.format("SELECT * FROM sdq_summary_view %s", whereClause(conditions)))
        .params(params)
        .query(
            (rs, rowNum) -> {
              UUID clientId = rs.getObject("client_id", UUID.class);
              LocalDate periodDate = rs.getDate("period_date").toLocalDate();
              String categoryJson = rs.getString("category_subtotals");
              String postureJson = rs.getString("posture_subtotals");
              Map<String, Integer> byCategory =
                  parseJson(categoryJson, new TypeReference<>() {}, Collections::emptyMap);
              Map<Posture, Integer> byPosture =
                  convertPosture(
                      parseJson(postureJson, new TypeReference<>() {}, Collections::emptyMap));

              int totalDifficulties = rs.getInt("total_difficulties");

              return SdqSubmissionSummary.builder()
                  .clientId(clientId)
                  .period(periodDate)
                  .assessor(
                      Optional.ofNullable(rs.getString("assessor"))
                          .map(Assessor::valueOf)
                          .orElse(Assessor.Unknown))
                  .categorySubTotals(byCategory)
                  .postureSubTotals(byPosture)
                  .totalDifficulties(totalDifficulties)
                  .build();
            })
        .list();
  }

  @Override
  public SdqSubmissionSummary getSummary(UUID periodId, Assessor assessor) {
    return jdbcClient
        .sql(
            "SELECT * from sdq_single_summary_view WHERE period_id = :periodId AND assessor = :assessor")
        .param("periodId", periodId)
        .param("assessor", assessor.name())
        .query(
            (rs, rowNum) -> {
              String categoryJson = rs.getString("category_subtotals");
              String postureJson = rs.getString("posture_subtotals");
              Map<String, Integer> byCategory =
                  parseJson(categoryJson, new TypeReference<>() {}, Collections::emptyMap);
              Map<Posture, Integer> byPosture =
                  convertPosture(
                      parseJson(postureJson, new TypeReference<>() {}, Collections::emptyMap));

              int totalDifficulties = rs.getInt("total_difficulties");

              return SdqSubmissionSummary.builder()
                  .categorySubTotals(byCategory)
                  .postureSubTotals(byPosture)
                  .totalDifficulties(totalDifficulties)
                  .build();
            })
        .single();
  }

  private static <T> Map<Posture, T> convertPosture(Map<String, T> raw) {
    Map<Posture, T> result = new EnumMap<>(Posture.class);
    raw.forEach((k, v) -> result.put(Posture.valueOf(k), v));
    return result;
  }

  private List<SdqScore> getScores(UUID periodId, Assessor assessor) {
    return jdbcClient
        .sql(
            """
          SELECT
            s.statement AS statement_key,
            s.score as score
          FROM
            sdq s
          INNER JOIN sdq_statement st
          ON s.statement = st.statement_key
          WHERE period_id = :periodId AND assessor = :assessor
        """)
        .param("periodId", periodId)
        .param("assessor", assessor.name())
        .query(
            (rs, rowNum) ->
                SdqScore.builder()
                    .statement(statementRepository.getStatement(rs.getString("statement_key")))
                    .score(rs.getInt("score"))
                    .build())
        .list();
  }
}
