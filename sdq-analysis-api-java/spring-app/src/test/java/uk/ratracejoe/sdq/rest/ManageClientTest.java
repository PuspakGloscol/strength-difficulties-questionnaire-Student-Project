package uk.ratracejoe.sdq.rest;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.InstanceOfAssertFactories.list;

import java.time.LocalDate;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.test.web.server.LocalServerPort;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.web.client.HttpClientErrorException;
import org.springframework.web.client.RestClient;
import uk.ratracejoe.sdq.SdqApiClient;
import uk.ratracejoe.sdq.SdqFixtures;
import uk.ratracejoe.sdq.dto.ClientQueryDTO;
import uk.ratracejoe.sdq.model.ParsedFile;
import uk.ratracejoe.sdq.model.SdqClient;
import uk.ratracejoe.sdq.model.demographics.*;

@SpringBootTest(webEnvironment = SpringBootTest.WebEnvironment.RANDOM_PORT)
@ActiveProfiles("test")
class ManageClientTest {
  private SdqApiClient client;
  private SdqFixtures fixtures;
  @LocalServerPort int port;

  @BeforeEach
  void beforeAll() {
    fixtures = new SdqFixtures(port);
    client = fixtures.getSdqClient();
  }

  @Test
  void deleteClient() {
    // Given
    List<ParsedFile> files = fixtures.givenAllTestFilesIngested();
    UUID clientIdToDelete = files.getFirst().sdqClient().clientId();

    // When
    List<SdqClient> clientsBeforeDelete = client.getAllClients();
    client.deleteClient(clientIdToDelete);
    List<SdqClient> clientsAfterDelete = client.getAllClients();

    // Then
    assertThat(clientsBeforeDelete).extracting(SdqClient::clientId).contains(clientIdToDelete);
    assertThat(clientsAfterDelete).extracting(SdqClient::clientId).doesNotContain(clientIdToDelete);
  }

  @Test
  void updateClient() {
    LocalDate dob = LocalDate.now().minusYears(20);
    SdqClient toCreate =
        SdqClient.builder()
            .codeName("Dave Lister")
            .dateOfBirth(dob)
            .council(Council.CHELTENHAM)
            .careExperience(CareExperience.YES_ADOPTED)
            .disabilityTypes(List.of(DisabilityType.LEARNING))
            .disabilityStatus(DisabilityStatus.DISABILITY)
            .fundingSource(FundingSource.EHCP)
            .interventions(
                List.of(
                    new Intervention(InterventionType.IA, 2),
                    new Intervention(InterventionType.CCPT, 3)))
            .aces(Map.of(AceType.GENERIC, 2))
            .build();
    SdqClient sdqClient = this.client.createClient(toCreate);
    assertThat(sdqClient)
        .extracting(SdqClient::interventions, list(Intervention.class))
        .extracting(Intervention::type)
        .containsExactlyInAnyOrder(InterventionType.IA, InterventionType.CCPT);

    SdqClient toUpdate =
        SdqClient.builder()
            .clientId(sdqClient.clientId())
            .codeName("Arnold Rimmer")
            .careExperience(CareExperience.KINSHIP)
            .disabilityTypes(List.of(DisabilityType.COGNITIVE_OR_MEMORY))
            .interventions(
                List.of(
                    new Intervention(InterventionType.IA, 2),
                    new Intervention(InterventionType.PTP, 3)))
            .build();
    SdqClient updated = this.client.updateClient(toUpdate);
    SdqClient afterUpdate = this.client.getClient(sdqClient.clientId());

    assertThat(updated).isEqualTo(afterUpdate);
    assertThat(updated).extracting(SdqClient::fundingSource).isEqualTo(FundingSource.EHCP);
    assertThat(updated).extracting(SdqClient::careExperience).isEqualTo(CareExperience.KINSHIP);
    assertThat(updated)
        .extracting(SdqClient::interventions, list(Intervention.class))
        .extracting(Intervention::type)
        .containsExactlyInAnyOrder(InterventionType.IA, InterventionType.PTP);
  }

  @Test
  void ignoresUnsupportedMultiValueDemographicFilters() {
    fixtures.givenAllTestFilesIngested();

    List<SdqClient> results =
        client.searchClients(
            ClientQueryDTO.builder()
                .filters(
                    List.of(
                        new DemographicFilter(DemographicField.DisabilityType, List.of("LEARNING")),
                        new DemographicFilter(DemographicField.ACES, List.of("GENERIC")),
                        new DemographicFilter(DemographicField.InterventionType, List.of("IA"))))
                .build());

    assertThat(results).hasSize(8);
  }

  @Test
  void rejectsUnsupportedMultiValueDemographicReports() {
    RestClient restClient = RestClient.builder().baseUrl("http://localhost:" + port).build();

    org.assertj.core.api.Assertions.assertThatThrownBy(
            () ->
                restClient
                    .get()
                    .uri("/api/client/demographic_report/DisabilityType")
                    .retrieve()
                    .toBodilessEntity())
        .isInstanceOf(HttpClientErrorException.BadRequest.class);
  }
}
