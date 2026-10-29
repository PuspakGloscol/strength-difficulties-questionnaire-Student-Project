package uk.ratracejoe.sdq.service;

import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.function.Function;
import java.util.stream.Collectors;
import lombok.RequiredArgsConstructor;
import uk.ratracejoe.sdq.exception.SdqException;
import uk.ratracejoe.sdq.model.EnumValue;
import uk.ratracejoe.sdq.model.demographics.*;
import uk.ratracejoe.sdq.model.gbo.GoalType;
import uk.ratracejoe.sdq.model.sdq.Category;
import uk.ratracejoe.sdq.model.sdq.Statement;
import uk.ratracejoe.sdq.repository.StatementRepository;

@RequiredArgsConstructor
public class RefDataServiceImpl implements RefDataService {
  private List<Category> categories;
  private List<Statement> statements;
  private Map<String, Statement> byKey;

  private final StatementRepository repository;

  @Override
  public Map<DemographicField, List<EnumValue>> getDemographicOptions() throws SdqException {
    return Map.of(
        DemographicField.Gender,
        Arrays.stream(Gender.values()).map(Gender::enumValue).toList(),
        DemographicField.Council,
        Arrays.stream(Council.values()).map(Council::enumValue).toList(),
        DemographicField.Ethnicity,
        Arrays.stream(Ethnicity.values()).map(Ethnicity::enumValue).toList(),
        DemographicField.EAL,
        Arrays.stream(EnglishAsAdditionalLanguage.values())
            .map(EnglishAsAdditionalLanguage::enumValue)
            .toList(),
        DemographicField.DisabilityStatus,
        Arrays.stream(DisabilityStatus.values()).map(DisabilityStatus::enumValue).toList(),
        DemographicField.CareExperience,
        Arrays.stream(CareExperience.values()).map(CareExperience::enumValue).toList(),
        DemographicField.FundingSource,
        Arrays.stream(FundingSource.values()).map(FundingSource::enumValue).toList());
  }

  @Override
  public List<Statement> getStatements() {
    ensure();
    return statements;
  }

  @Override
  public List<Category> getCategories() {
    ensure();
    return categories;
  }

  @Override
  public Statement getStatement(String key) {
    ensure();
    return byKey.get(key);
  }

  @Override
  public List<EnumValue> getGoalTypes() {
    return Arrays.stream(GoalType.values()).map(GoalType::enumValue).toList();
  }

  private void ensure() {
    if (Objects.nonNull(statements)) return;

    categories = repository.getCategories();
    statements = repository.getStatements();
    byKey = statements.stream().collect(Collectors.toMap(Statement::key, Function.identity()));
  }
}
