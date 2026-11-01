package uk.ratracejoe.sdq.config;

import java.io.IOException;
import org.jspecify.annotations.NonNull;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.io.Resource;
import org.springframework.web.servlet.config.annotation.ResourceHandlerRegistry;
import org.springframework.web.servlet.config.annotation.WebMvcConfigurer;
import org.springframework.web.servlet.resource.PathResourceResolver;

@Configuration
public class SpaConfig implements WebMvcConfigurer {
  public static class SpaPageResolver extends PathResourceResolver {
    @Override
    protected Resource getResource(@NonNull String resourcePath, Resource location)
        throws IOException {
      Resource requested = location.createRelative(resourcePath);

      // If the file exists, serve it
      if (requested.exists() && requested.isReadable()) {
        return requested;
      }

      // Otherwise, serve index.html
      return location.createRelative("index.html");
    }
  }

  @Override
  public void addResourceHandlers(ResourceHandlerRegistry registry) {
    registry
        .addResourceHandler("/**")
        .addResourceLocations("classpath:/static/")
        .resourceChain(true)
        .addResolver(new SpaPageResolver());
  }
}
