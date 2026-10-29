package uk.ratracejoe.sdq.repository;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.AssertionsForClassTypes.tuple;

import java.time.LocalDate;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.test.web.server.LocalServerPort;
import org.springframework.test.context.ActiveProfiles;
import uk.ratracejoe.sdq.SdqApiClient;
import uk.ratracejoe.sdq.SdqFixtures;
import uk.ratracejoe.sdq.dto.GoalQueryDTO;
import uk.ratracejoe.sdq.model.Assessor;
import uk.ratracejoe.sdq.model.SdqClient;
import uk.ratracejoe.sdq.model.demographics.DemographicField;
import uk.ratracejoe.sdq.model.demographics.DemographicFilter;
import uk.ratracejoe.sdq.model.demographics.Gender;
import uk.ratracejoe.sdq.model.gbo.Goal;
import uk.ratracejoe.sdq.model.gbo.GoalProgress;
import uk.ratracejoe.sdq.model.gbo.GoalType;

@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
@ActiveProfiles("test")
class GoalAnalyticTests {

  private SdqFixtures fixtures;
  private SdqApiClient client;

  @LocalServerPort int port;

  @BeforeEach
  void beforeEach() {
    fixtures = new SdqFixtures(port);
    client = fixtures.getSdqClient();
  }

  @Test
  void getGoalWithType() {
    // Given
    SdqClient sdqClient = client.ingestFile("Test File 4.xlsx").getBody().getFirst().sdqClient();

    // When
    List<Goal> goals = client.getGoalsForClient(sdqClient.clientId());

    // Then
    assertThat(goals)
        .extracting(Goal::type)
        .containsExactly(
            GoalType.EMOTIONAL,
            GoalType.BEHAVIOURAL,
            GoalType.TRAUMA_RECOVERY,
            GoalType.SELF_ESTEEM_CONFIDENCE);
  }

  @Test
  void getGoalsWithProgress() {
    fixtures.givenAllTestFilesIngested();

    List<GoalProgress> result =
        client
            .getGoalsWithProgress(
                GoalQueryDTO.builder()
                    .assessors(List.of(Assessor.Parent1))
                    .filters(
                        List.of(
                            new DemographicFilter(
                                DemographicField.Gender, List.of(Gender.FEMALE.name()))))
                    .minProgress(3)
                    .from(LocalDate.of(2024, 5, 1))
                    .to(LocalDate.of(2025, 11, 1))
                    .build())
            .getBody();
    assertThat(result).hasSize(3);
    assertThat(result).extracting(GoalProgress::assessor).containsOnly(Assessor.Parent1);
    assertThat(result)
        .extracting(
            g -> g.goal().type(),
            g -> g.goal().description(),
            GoalProgress::firstScore,
            GoalProgress::lastScore)
        .containsExactlyInAnyOrder(
            tuple(GoalType.TRAUMA_RECOVERY, "Goal 2 for Test File 7.xlsx", 2, 7),
            tuple(GoalType.TRAUMA_RECOVERY, "Goal 1 for Test File 6.xlsx", 3, 8),
            tuple(GoalType.BEHAVIOURAL, "Goal 2 for Test File 6.xlsx", 2, 7));
  }

  @Test
  void getGoalsOfType() {
    fixtures.givenAllTestFilesIngested();

    List<GoalProgress> result =
        client
            .getGoalsWithProgress(
                GoalQueryDTO.builder()
                    .assessors(List.of(Assessor.School))
                    .minProgress(1)
                    .goalTypes(List.of(GoalType.BEHAVIOURAL, GoalType.EMOTIONAL))
                    .from(LocalDate.of(2024, 5, 1))
                    .to(LocalDate.of(2025, 11, 1))
                    .build())
            .getBody();
    assertThat(result)
        .extracting(GoalProgress::goal)
        .extracting(Goal::type)
        .containsOnly(GoalType.BEHAVIOURAL, GoalType.EMOTIONAL);
  }

  @Test
  void ignoresUnsupportedMultiValueDemographicFilters() {
    fixtures.givenAllTestFilesIngested();

    List<GoalProgress> result =
        client
            .getGoalsWithProgress(
                GoalQueryDTO.builder()
                    .assessors(List.of(Assessor.Parent1))
                    .filters(
                        List.of(
                            new DemographicFilter(
                                DemographicField.DisabilityType, List.of("LEARNING"))))
                    .minProgress(1)
                    .from(LocalDate.of(2024, 5, 1))
                    .to(LocalDate.of(2025, 11, 1))
                    .build())
            .getBody();

    assertThat(result).isNotEmpty();
  }
}
